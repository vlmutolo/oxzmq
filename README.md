# IMPORTANT

[Development](https://git.sr.ht/~vlmutolo/oxzmq) and [tracker](https://todo.sr.ht/~vlmutolo/oxzmq) moved to [SourceHut](https://sourcehut.org)

# OxZMQ (`oxzmq`)

OxZMQ is an implementation of [ZeroMQ][zmq] in pure Rust. Its high-level goals are as follows, ordered by priority.

1. Fool-proof, impossible-to-misuse API
2. As efficient as possible, hopefully one day on par with [`libzmq`][libzmq]
3. Match the functionality of `libzmq`

The name is pronounced "oxy-em-queue" because "oxy" is kind of like "oxygenation", which is kind of like Rust. 

[zmq]: https://zeromq.org/
[libzmq]: https://github.com/zeromq/libzmq

## License — [MPL 2.0][mpl2-text]
This project is licensed under the MPL, Version 2 ([license FAQ here][mpl2-faq]). I chose the MPL in spite of the fact that Rust projects overwhelmingly use a duel MIT/Apache-2 license. However, it is a [non-viral license][virality], so hopefully members of the Rust community won't be afraid to use it in their projects.

Reasons I chose the MPL:

- It ensures that all changes made to downstream forks can be reintegrated into the project.
- It isn't viral—you can license "larger works" under an arbitrary license.
- I was swayed by the arguments Peter Hintjens made in the [ZeroMQ guide][zmq-guide].
- The ZeroMQ project generally intends to move to the MPL and [recommends it for all new projects][zmq-mpl].

[mpl2-text]: https://www.mozilla.org/en-US/MPL/2.0/
[mpl2-faq]: https://www.mozilla.org/en-US/MPL/2.0/FAQ/
[virality]: https://www.mozilla.org/en-US/MPL/2.0/FAQ/#virality
[zmq-guide]: http://zguide.zeromq.org/page:all#Chapter-The-ZeroMQ-Community
[zmq-mpl]: http://wiki.zeromq.org/area:licensing

## Project Maintenance — C4
In the spirit of ZeroMQ, this project will do its best to adhere to the principles of C4. The exact protocol for C4 is described in its [specification][c4-spec]. This has a few consequences worth mentioning here:

- Anyone who submits an accepted patch can become a maintainer.
- We will only have a single branch on this repo, which we'll call "master".
- No one, without exception, can merge their own patch. This will take effect one there's more than a single maintainer on the project.
- All patches should solve exactly one issue, and that issue should have a clear statement of a real and current problem identified with the project.

The C4 specification has more detailed information, and the [ZeroMQ Guide][zmq-guide] gives justifications for the process.

[c4-spec]: https://rfc.zeromq.org/spec/42/
[zmq-guide]: http://zguide.zeromq.org/page:all#Chapter-The-ZeroMQ-Community
