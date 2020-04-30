/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use futures::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite};
use std::convert::TryFrom;

const MORE_FLAG_IDX: u8 = 0;
const LONG_FLAG_IDX: u8 = 1;
const KIND_FLAG_IDX: u8 = 2;

const SHORT_SIZE_LEN: usize = 1;
const LONG_SIZE_LEN: usize = 8;

#[derive(Clone, Debug)]
pub enum Frame {
    Command(CommandFrame),
    Message(MessageFrame),
}

#[derive(Clone, Debug)]
pub struct CommandFrame {
    pub(crate) name: String,
    pub(crate) data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct MessageFrame {
    more: bool,
    data: Vec<u8>,
}

impl Frame {
    pub(crate) fn new_command(cmd_name: String, data: Vec<u8>) -> Frame {
        Frame::Command(CommandFrame {
            name: cmd_name,
            data,
        })
    }

    pub(crate) fn new_message(more: bool, data: Vec<u8>) -> Frame {
        Frame::Message(MessageFrame { more, data })
    }

    pub(crate) fn data(&self) -> &[u8] {
        match self {
            Frame::Command(cmd) => cmd.data.as_slice(),
            Frame::Message(msg) => msg.data.as_slice(),
        }
    }

    /// This creates a "fatal error" command from an error message, truncating
    /// the length of the message at 255 characters.
    pub(crate) fn new_fatal_error(msg: &str) -> Frame {
        // Truncate the message to a max of 255 bytes.
        let mut final_byte_idx = 0_usize;
        for c in msg.chars() {
            if final_byte_idx + c.len_utf8() > u8::max_value() as usize {
                break;
            }
            final_byte_idx += c.len_utf8();
        }

        // Technically indexing like this can panic if the index isn't on the
        // boundary of a `char`, but it never should if we did the previous
        // truncation correctly. This is why we iterated over `char`s.
        let msg = &msg[..final_byte_idx];

        // This should never fail because we just
        // truncated the value to under u8::max_value() bytes.
        let msg_size = u8::try_from(msg.len()).unwrap();

        let mut data: Vec<u8> = Vec::with_capacity(1 + msg.len());
        data.push(msg_size);
        data.extend_from_slice(msg.as_bytes());

        Frame::Command(CommandFrame {
            name: "ERROR".to_string(),
            data,
        })
    }

    pub(crate) async fn read_new<R: AsyncBufRead + Unpin>(
        stream: &mut R,
    ) -> Result<Frame, FrameParseError> {
        let mut flags_buf = [0_u8; 1];
        stream.read_exact(&mut flags_buf).await?;
        let flag_bits = u8::from_be_bytes(flags_buf);

        let more_frames = get_bit(flag_bits, MORE_FLAG_IDX);

        let long = get_bit(flag_bits, LONG_FLAG_IDX);

        let kind = match get_bit(flag_bits, KIND_FLAG_IDX) {
            true => FrameKind::Command,
            false => FrameKind::Message,
        };

        // Bits 3–7 inclusive shall not be set (according to the spec).
        for bit in 3..8 {
            if get_bit(flag_bits, bit) {
                return Err(FrameParseError::Flags);
            }
        }

        let data_len: u64 = if long {
            let mut len_buf = [0_u8; LONG_SIZE_LEN];
            stream.read_exact(&mut len_buf).await?;
            u64::from_be_bytes(len_buf)
        } else {
            let mut len_buf = [0_u8; SHORT_SIZE_LEN];
            stream.read_exact(&mut len_buf).await?;
            u8::from_be_bytes(len_buf) as u64
        };
        let data_len =
            usize::try_from(data_len).map_err(|e| FrameParseError::MessageTooLarge(e))?;

        let frame = match kind {
            FrameKind::Command => {
                if more_frames {
                    return Err(FrameParseError::MultipartCommand);
                }

                // Read the command name.
                let mut command_name_bytes = Vec::<u8>::with_capacity(10);
                stream.read_until(0x00, &mut command_name_bytes).await?;

                // Get rid of the null delimiter.
                command_name_bytes.pop();
                let command_name = String::from_utf8(command_name_bytes)?;

                let mut command_data = Vec::new();
                stream.read_to_end(&mut command_data).await?;

                Frame::Command(CommandFrame {
                    name: command_name,
                    data: command_data,
                })
            }
            FrameKind::Message => {
                let mut message_data = Vec::with_capacity(data_len);
                stream.read_to_end(&mut message_data).await?;
                Frame::Message(MessageFrame {
                    more: more_frames,
                    data: message_data,
                })
            }
        };

        Ok(frame)
    }

    pub(crate) async fn write_to<W: AsyncWrite + Unpin>(
        &self,
        stream: &mut W,
    ) -> Result<(), io::Error> {
        let mut flags = 0_u8;
        match self {
            Frame::Message(msg_frame) if msg_frame.more => flags = set_bit(flags, MORE_FLAG_IDX),
            _ => (),
        }

        if self.data().len() > u8::max_value() as usize {
            flags = set_bit(flags, LONG_FLAG_IDX);
        }
        if let Frame::Command(_) = self {
            flags = set_bit(flags, KIND_FLAG_IDX);
        }
        let flags = flags; // make immutable

        // Account for the length of the command name, which technically goes in the
        // "data" field for the frame.
        let total_data_len = if let Frame::Command(cmd) = self {
            self.data().len() + cmd.name.len()
        } else {
            self.data().len()
        };

        // The length can either be encoded as 1 or 8 bytes.
        let length_bytes_len = if total_data_len > u8::max_value() as usize {
            LONG_SIZE_LEN
        } else {
            SHORT_SIZE_LEN
        };
        let length_bytes = &self.data().len().to_be_bytes()[..length_bytes_len];


        // Create a buffer to hold some small intermediate writes. We probably need no
        // more than 20 bytes because flags=1, length<=8, and name is usually <= 5.
        let mut pre_data_buf: Vec<u8> = Vec::with_capacity(20);

        pre_data_buf.push(flags);
        pre_data_buf.extend_from_slice(length_bytes);

        // If the frame is a command, send the command name and a null separator
        // before the command data.
        if let Frame::Command(cmd) = self {
            pre_data_buf.extend_from_slice(&cmd.name.as_bytes());
            pre_data_buf.push(0x00);
        }

        io::copy(pre_data_buf.as_slice(), stream).await?;
        io::copy(self.data(), stream).await?;

        Ok(())
    }
}

// Returns `false` for out-of-range gets
fn get_bit(n: u8, bit: u8) -> bool {
    if bit < 8 {
        n & (1 << bit) != 0
    } else {
        false
    }
}

// Discards sets to out-of-range sets
fn set_bit(n: u8, bit: u8) -> u8 {
    if bit < 8 {
        n | (1 << bit)
    } else {
        n
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FrameParseError {
    #[error("error reading data stream")]
    Io(#[from] io::Error),

    #[error("malformed flags")]
    Flags,

    #[error("Command frames cannot be multipart")]
    MultipartCommand,

    #[error("command name must be valid utf-8")]
    CommandNameInvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("msg size indicates msg is too large to fit in memory")]
    MessageTooLarge(std::num::TryFromIntError),
}

#[derive(Clone, Debug)]
pub(crate) enum FrameKind {
    Command,
    Message,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_bit() {
        let u = 0b_1001_0001;
        assert_eq!(get_bit(n, 0), true);
        assert_eq!(get_bit(n, 1), false);
        assert_eq!(get_bit(n, 4), true);
        assert_eq!(get_bit(n, 7), true);
        assert_eq!(get_bit(n, 8), false);
    }
}
