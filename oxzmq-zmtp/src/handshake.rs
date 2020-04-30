/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{
    handshake::null::{NullHandshake, NullHandshakeError},
    socket::SocketType,
    Greeting, Mechanism,
};
use futures::io::{self, AsyncBufRead, AsyncRead, AsyncWrite};
use std::{collections::HashMap, convert::TryFrom};

mod null;

#[derive(Debug, Clone)]
pub(crate) enum Handshake {
    Null(NullHandshake),
}

impl Handshake {
    pub(crate) async fn perform<S>(
        stream: &mut S,
        greeting: &Greeting,
        socket_type: &SocketType,
    ) -> Result<Handshake, HandshakeError>
    where
        S: AsyncWrite + AsyncRead + AsyncBufRead + Unpin,
    {
        match greeting.mechanism {
            Mechanism::Null => Ok(Handshake::Null(
                NullHandshake::perform(stream, socket_type).await?,
            )),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HandshakeError {
    #[error("error in handshake with NULL mechanism")]
    Null(#[from] NullHandshakeError),
}

#[derive(Debug, Clone)]
pub(crate) struct Properties {
    inner: HashMap<String, Vec<u8>>,
}

impl Properties {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    // More info: https://rfc.zeromq.org/spec/23/#the-null-security-mechanism
    fn parse_from_slice(bytes: &[u8]) -> Result<Self, PropertiesParseError> {
        let mut map = HashMap::<String, Vec<u8>>::new();

        let mut rest = bytes;
        while rest.len() > 0 {
            let name_size = *rest.get(0).ok_or(PropertiesParseError::EmptySlice)? as usize;
            if name_size == 0 {
                return Err(PropertiesParseError::ZeroSizedName);
            }
            rest = &rest[1..];
            if rest.len() < name_size as usize {
                return Err(PropertiesParseError::NameSizeIncorrect);
            }

            let name = std::str::from_utf8(&rest[..name_size])
                .map_err(|_| PropertiesParseError::NameInvalidChar)?;
            if !name
                .chars()
                .all(|c| c.is_alphanumeric() && ['-', '_', '.', '+'].contains(&c))
            {
                return Err(PropertiesParseError::NameInvalidChar);
            }
            rest = &rest[name_size..];

            let value_size_bytes = <[u8; 4]>::try_from(&rest[..4])
                .map_err(|_| PropertiesParseError::ValueSizeIncomplete)?;
            let value_size = u32::from_be_bytes(value_size_bytes) as usize;
            rest = &rest[4..];
            if rest.len() < value_size as usize {
                return Err(PropertiesParseError::ValueSizeIncorrect);
            }
            let value_bytes = &rest[..value_size];

            map.insert(name.to_lowercase(), value_bytes.to_vec());
        }

        Ok(Properties { inner: map })
    }

    async fn write_to<W: AsyncWrite + Unpin>(&self, stream: &mut W) -> Result<(), io::Error> {
        let mut write_buf = Vec::<u8>::new();

        for (name, value) in self.inner.iter() {
            let name_size_bytes = name.len().to_be_bytes();
            write_buf.extend_from_slice(&name_size_bytes);
            write_buf.extend_from_slice(name.as_bytes());

            let value_size_bytes = value.len().to_be_bytes();
            write_buf.extend_from_slice(&value_size_bytes);
            write_buf.extend_from_slice(value.as_slice());
        }

        io::copy(write_buf.as_slice(), stream).await?;

        Ok(())
    }

    // We `get` keys through a method because we have to ensure that we treat
    // all keys as lowercase.
    pub(crate) fn get(&self, key: String) -> Option<&[u8]> {
        self.inner.get(&key.to_lowercase()).map(|v| v.as_slice())
    }

    // We `insert` keys through a method because we have to ensure that we treat
    // all keys as lowercase.
    fn insert(&mut self, key: String, value: Vec<u8>) {
        self.inner.insert(key.to_lowercase(), value);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PropertiesParseError {
    #[error("given slice was empty")]
    EmptySlice,

    #[error("name had size of zero")]
    ZeroSizedName,

    #[error("invalid character(s) in property name")]
    NameInvalidChar,

    #[error("name size indicated more bytes than were available")]
    NameSizeIncorrect,

    #[error("not enough bytes left to read size of metadata value")]
    ValueSizeIncomplete,

    #[error("value size indicated more bytes than were available")]
    ValueSizeIncorrect,
}
