/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite};

#[derive(Debug, Clone)]
struct ZmtpSocket {
    connections: Vec<Connection>,
}

#[derive(Debug, Clone)]
struct Connection {
    greeting: Greeting,
    handshake: Handshake,
}

impl<S: AsyncRead + AsyncWrite> Connection {
    pub async fn greeting(stream: S) -> Result<Greeting, GreetingError> {
        
    }
}

struct Greeting {
    version: Version,
    mechanism: Mechanism,
    as_server: AsServer,
}

#[derive(thiserror::Error, Debug, Clone)]
struct GreetingError {}

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
