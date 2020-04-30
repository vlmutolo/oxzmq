<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. -->

# Deviations from ZeroMQ Expectations

OxZMQ will inevitably deviate from what people expect from a ZeroMQ implementation. This will happen for two reasons.

1. During development, we will not be able to support all the features of `libzmq` and its dependencies. This is unavoidable, but the situation will improve over time.
2. This project may make opinionated design decisions that differ from the original ZeroMQ project. These will be kept to a minimum, will ideally only apply to the high-level Rust API to make it idiomatic, and will all be documented here.

## ZMTP

### The `socket-type` field must be specified.
The `oxzmq-zmtp` library requires that clients specify the `socket-type` property in the NULL handshake. The authors didn't know how to work around this, so this is the current behavior. The specification says that implementations "SHOULD" specify the property, but does not require that they do so. If anyone knows the correct way to deal with a missing `socket-type` property, please file an issue and we will fix it.

### Messages cannot be multiplexed.
I don't know if this is a hard requirement of the original protocol, but currently `oxzmq-zmtp` assumes that messages will only ever be sent one at a time. This means, for example, that a peer won't start sending a multipart message and send a command in the middle of it, intermixed with the message. It also means that a peer won't intersperse different parts of different multipart messages.. Again, if this assumption is bad, please file an issue and we'll fix it. I'm making the assumption because it greatly simplifies the implementation.