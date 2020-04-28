/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryFrom;

const SUPPORTED_SOCKET_TYPES: [SocketType; 2] = [SocketType::Req, SocketType::Rep];

#[derive(Clone, Debug)]
pub(crate) enum SocketType {
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
    fn valid_socket_combo(&self, other: &SocketType) -> bool {
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

impl<B: AsRef<[u8]>> TryFrom<B> for SocketType {
    type Error = SocketTypeFromStrError;

    fn from(s: S) -> SocketType {
        let socket_type = match s {
            b"REQ" => SocketType::Req,
            b"REP" => SocketType::Rep,
            b"DEALER" => SocketType::Dealer,
            b"ROUTER" => SocketType::Router,
            b"PUB" => SocketType::Pub,
            b"SUB" => SocketType::Sub,
            b"XPUB" => SocketType::XPub,
            b"XSUB" => SocketType::XSub,
            b"PUSH" => SocketType::Push,
            b"PULL" => SocketType::Pull,
            b"PAIR" => SocketType::Pair,
            b => return Err(SocketTypeFromStrError::Unknown(b.to_vec())),
        };

        if !SUPPORTED_SOCKET_TYPES.contains(socket_type) {
            return Err(SocketTypeFromStrError::Unsupported(s.to_string()));
        }

        socket_type
    }
}

impl From<&SocketType> for &'static str {
    fn from(socket: SocketType) -> &'static str {
        match s {
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
    fn from(socket: SocketType) -> String {
        <&str>::from(socket_type).to_string()
    }
}

#[derive(thiserror::Error, Debug)]
enum SocketTypeFromStrError {
    #[error("unknown socket type: {0}")]
    Unknown(String),

    #[error("unsupported socket type: {0}")]
    Unsupported(String),
}
