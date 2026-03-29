use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct ChatMessage {
    pub msg_type: u16,
    pub text: String,
}

impl Encodable for ChatMessage {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_smart(self.msg_type);
        buf.put_u32(0);
        buf.put_u8(0);
        buf.put_string(&self.text);

        Frame {
            opcode: 26,
            prefix: Prefix::Byte,
            payload: buf.freeze(),
        }
    }
}
