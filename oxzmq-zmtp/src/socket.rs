/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryFrom;

const SUPPORTED_SOCKET_TYPES: [SocketType; 2] = [SocketType::Req, SocketType::Rep];

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum SocketType {
    Req,
    Rep,
    Dealer,
    Router,
    Pub,
    Sub,
    XPub,
    XSub,
    Push,
    Pull,
    Pair,
}

impl SocketType {
    pub(crate) fn valid_socket_combo(&self, other: &SocketType) -> bool {
        match self {
            SocketType::Req => [SocketType::Rep, SocketType::Router].contains(other),
            SocketType::Rep => [SocketType::Req, SocketType::Dealer].contains(other),
            SocketType::Dealer => {
                [SocketType::Rep, SocketType::Dealer, SocketType::Router].contains(other)
            }
            SocketType::Router => {
                [SocketType::Req, SocketType::Dealer, SocketType::Router].contains(other)
            }
            SocketType::Pub => [SocketType::Sub, SocketType::XSub].contains(other),
            SocketType::XPub => [SocketType::Sub, SocketType::XSub].contains(other),
            SocketType::Sub => [SocketType::Pub, SocketType::XPub].contains(other),
            SocketType::XSub => [SocketType::Pub, SocketType::XPub].contains(other),
            SocketType::Push => [SocketType::Pull].contains(other),
            SocketType::Pull => [SocketType::Push].contains(other),
            SocketType::Pair => [SocketType::Pair].contains(other),
        }
    }
}

impl TryFrom<&[u8]> for SocketType {
    type Error = SocketTypeFromBytesError;

    fn try_from(bytes: &[u8]) -> Result<SocketType, SocketTypeFromBytesError> {
        let socket_name = std::str::from_utf8(bytes)?;
        let socket_type = match socket_name {
            "REQ" => SocketType::Req,
            "REP" => SocketType::Rep,
            "DEALER" => SocketType::Dealer,
            "ROUTER" => SocketType::Router,
            "PUB" => SocketType::Pub,
            "SUB" => SocketType::Sub,
            "XPUB" => SocketType::XPub,
            "XSUB" => SocketType::XSub,
            "PUSH" => SocketType::Push,
            "PULL" => SocketType::Pull,
            "PAIR" => SocketType::Pair,
            s => return Err(SocketTypeFromBytesError::Unknown(s.to_string())),
        };

        if !SUPPORTED_SOCKET_TYPES.contains(&socket_type) {
            return Err(SocketTypeFromBytesError::Unsupported(socket_name.to_string()));
        }

        Ok(socket_type)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SocketTypeFromBytesError {
    #[error("unknown socket type: {0}")]
    Unknown(String),

    #[error("known-but-unsupported socket type: {0}")]
    Unsupported(String),

    #[error("socket bytes were invalid utf8")]
    NotUtf8(#[from] std::str::Utf8Error),
}

impl From<&SocketType> for &'static str {
    fn from(socket: &SocketType) -> &'static str {
        match socket {
            SocketType::Req => "REQ",
            SocketType::Rep => "REP",
            SocketType::Dealer => "DEALER",
            SocketType::Router => "ROUTER",
            SocketType::Pub => "PUB",
            SocketType::Sub => "SUB",
            SocketType::XPub => "XPUB",
            SocketType::XSub => "XSUB",
            SocketType::Push => "PUSH",
            SocketType::Pull => "PULL",
            SocketType::Pair => "PAIR",
        }
    }
}

impl From<&SocketType> for String {
    fn from(socket_type: &SocketType) -> String {
        <&str>::from(socket_type).to_string()
    }
}
