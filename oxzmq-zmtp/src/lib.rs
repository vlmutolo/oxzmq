/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    frame::{Frame, FrameParseError, MessageFrame},
    handshake::Handshake,
    socket::SocketType,
};
use futures::io::{self, AsyncRead, AsyncReadExt, AsyncWrite};
use std::marker::Unpin;

mod frame;
mod handshake;
mod socket;

const PADDING_LEN: usize = 80;
const FILLER_LEN: usize = 31;
const GREETING_BUF_LEN: usize = 100;

#[derive(Debug, Clone)]
pub struct ZmtpSocket<S> {
    connections: Vec<Connection<S>>,
    socket_type: SocketType,
}

#[derive(Debug, Clone)]
struct Connection<S> {
    remote_version: Version,
    remote_socket_type: SocketType,
    multipart_buffer: Vec<MessageFrame>,
    stream: S,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub async fn new(
        mut stream: S,
        socket_type: &SocketType,
    ) -> Result<Connection<S>, ConnectionError> {
        let greeting = Greeting::read_new(&mut stream).await?;
        let remote_version = greeting.version;

        // TODO: Send error here if remote_version isn't supported.

        let handshake = Handshake::perform(&mut stream, &greeting, &socket_type).await?;

        let remote_socket_type_bytes = match handshake {
            Handshake::Null(null_handshake) => null_handshake.properties.get("socket-type"),
        };
        let remote_socket_type = SocketType::from(remote_socket_type_bytes);

        // Check if the socket types are a valid combination.
        if !socket_type.valid_socket_combo(remote_socket_type) {
            let err_cmd = Frame::new_fatal_error("invalid socket combination");
            err_cmd.write_to(&mut stream).await?;
            return Err(ConnectionError::InvalidSocketCombination((
                socket_type,
                remote_socket_type,
            )));
        }

        Ok(Self {
            remote_version,
            remote_socket_type,
            stream,
        })
    }

    pub async fn recv_frame(&mut self) -> Result<Frame, RecvFrameError> {
        Ok(Frame::read_new(self.stream).await?)
    }
}

#[derive(thiserror::Error, Debug)]
enum ConnectionError {
    #[error("error reading data stream")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Greeting(#[from] GreetingError),

    #[error("invalid socket combination: {} with {}")]
    InvalidSocketCombination((SocketType, SocketType)),
}

#[derive(thiserror::Error, Debug)]
enum RecvFrameError {
    #[error("error reading data stream")]
    Io(#[from] io::Error),

    MalformedFrame(#[from] FrameParseError),
}

#[derive(Debug, Clone)]
struct Greeting {
    version: Version,
    mechanism: Mechanism,
    as_server: AsServer,
}

impl Greeting {
    pub async fn read_new<R>(stream: &mut R) -> Result<Greeting, GreetingError>
    where
        R: AsyncRead + Unpin,
    {
        // Read signature
        let mut sig_first_byte_buf = [0_u8; 1];
        let mut sig_padding_buf = [0_u8; PADDING_LEN];
        let mut sig_last_byte_buf = [0_u8; 1];

        stream.read_exact(&mut sig_first_byte_buf).await?;
        stream.read_exact(&mut sig_padding_buf).await?;
        stream.read_exact(&mut sig_last_byte_buf).await?;

        let sig_first_byte = u8::from_be_bytes(sig_first_byte_buf);
        let sig_last_byte = u8::from_be_bytes(sig_last_byte_buf);

        if sig_first_byte != 0xFF {
            return Err(GreetingError::Signature);
        }

        if sig_last_byte != 0x7F {
            return Err(GreetingError::Signature);
        }

        // Read version
        let mut version_major_buf = [0_u8; 1];
        let mut version_minor_buf = [0_u8; 1];

        stream.read_exact(&mut version_major_buf).await?;
        stream.read_exact(&mut version_minor_buf).await?;

        let version = Version {
            major: u8::from_be_bytes(version_major_buf),
            minor: u8::from_be_bytes(version_minor_buf),
        };

        // Read mechanism
        let mut mechanism_buf = [0_u8; 20];
        stream.read_exact(&mut mechanism_buf).await?;
        let null_idx = mechanism_buf
            .iter()
            .position(|&x| x == 0x00)
            .unwrap_or(mechanism_buf.len());
        let mechanism_str = std::str::from_utf8(&mechanism_buf[..null_idx])?;
        if mechanism_str.chars().any(|c| {
            c.is_lowercase() || !(c.is_alphanumeric() || ['-', '_', '.', '+'].contains(&c))
        }) {
            return Err(GreetingError::MechanismInvalidChar);
        }
        let mechanism = match mechanism_str {
            "NULL" => Mechanism::Null,
            _ => return Err(GreetingError::MechanismUnsupported),
        };

        // Read as-server
        let mut as_server_buf = [0_u8; 1];
        stream.read_exact(&mut as_server_buf).await?;
        let as_server = match as_server_buf {
            [0x00] => AsServer::Client,
            [0x01] => AsServer::Server,
            [x] => return Err(GreetingError::AsServer(x)),
        };

        // Read filler
        let mut filler_buf = [0_u8; FILLER_LEN];
        stream.read_exact(&mut filler_buf).await?;

        Ok(Self {
            version,
            mechanism,
            as_server,
        })
    }
}

#[derive(thiserror::Error, Debug)]
enum GreetingError {
    #[error("error reading data stream")]
    Io(#[from] io::Error),

    #[error("malformed signature")]
    Signature,

    #[error("unsupported version: {0:?}")]
    Version(Version),

    #[error("mechanism not utf8: {0}")]
    MechanismNotUtf8(#[from] std::str::Utf8Error),

    #[error("invalid character in mechanism string")]
    MechanismInvalidChar,

    #[error("mechanism string not supported")]
    MechanismUnsupported,

    #[error("invalid as-server value: {0}")]
    AsServer(u8),
}

#[derive(Debug, Clone)]
struct Version {
    major: u8,
    minor: u8,
}

#[derive(Debug, Clone)]
enum Mechanism {
    Null,
}

#[derive(Debug, Clone)]
enum AsServer {
    Server,
    Client,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
