/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    frame::{Frame, FrameParseError},
    handshake::{Properties, PropertiesParseError},
    socket::SocketType,
};
use futures::io::{self, AsyncBufRead, AsyncRead, AsyncWrite};

// More info: https://rfc.zeromq.org/spec/23/#the-null-security-mechanism#
#[derive(Debug, Clone)]
pub(crate) struct NullHandshake {
    pub(crate) properties: Properties,
}

impl NullHandshake {
    pub(crate) async fn perform<S>(
        stream: &mut S,
        socket_type: &SocketType,
    ) -> Result<NullHandshake, NullHandshakeError>
    where
        S: AsyncWrite + AsyncRead + AsyncBufRead + Unpin,
    {
        // As written in spec, send READY command first.
        let mut ready_cmd_data = Vec::new();
        let mut properties = Properties::new();
        properties.insert(
            "socket-type".to_string(),
            String::from(socket_type).into_bytes(),
        );
        properties.write_to(&mut ready_cmd_data).await?;

        let ready_cmd = Frame::new_command(String::from("READY"), ready_cmd_data);
        ready_cmd.write_to(stream).await?;

        // Receive and validate READY command frame.
        let received_frame = Frame::read_new(stream).await?;
        let received_cmd = match received_frame {
            Frame::Command(cmd) => cmd,
            Frame::Message(_) => return Err(NullHandshakeError::NoReadyCommand),
        };

        if received_cmd.name != "READY" {
            return Err(NullHandshakeError::NoReadyCommand);
        }

        let received_properties = Properties::parse_from_slice(received_cmd.data.as_slice())?;

        Ok(NullHandshake {
            properties: received_properties,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NullHandshakeError {
    #[error("error reading data stream")]
    Io(#[from] io::Error),

    #[error("peer did not send READY command")]
    NoReadyCommand,

    #[error("could not parse frame")]
    FrameParse(#[from] FrameParseError),

    #[error("could not parse properties")]
    PropertiesParse(#[from] PropertiesParseError),
}
