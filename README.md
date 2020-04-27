# OxZMQ

OxZMQ is an implementation of [ZeroMQ][zmq] in pure Rust. It's high-level goals are as follows, ordered by priority.

1. Fool-proof, impossible-to-misuse API
2. 100% memory-safe (`deny(unsafe)`)
3. As efficient as possible, hopefully one day matching [`libzmq`][libzmq]

[zmq](https://zeromq.org/)
[libzmq](https://github.com/zeromq/libzmq)

## License
This project is licensed under the MPL, Version 2. I chose the MPL in spite of the fact that Rust projects overwhelmingly use a duel MIT/Apache-2 license. However, it is a [non-viral license][virality], so hopefully members of the Rust community won't be afraid to use it in their projects.

Reasons I chose the MPL:

- It ensures that all changes made to downstream forks can be reintegrated into the project.
- It isn't viral in the sense that you can license "larger works" under an arbitrary license.
- I was swayed by the arguments Peter Hintjens made in the [ZeroMQ guide][zmq-guide].

[virality](https://www.mozilla.org/en-US/MPL/2.0/FAQ/#virality)
[zmq-guide](http://zguide.zeromq.org/page:all#Chapter-The-ZeroMQ-Community)