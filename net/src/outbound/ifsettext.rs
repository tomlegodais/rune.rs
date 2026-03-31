use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct IfSetText {
    pub parent: u16,
    pub component: u16,
    pub text: String,
}

impl Encodable for IfSetText {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        let hash = ((self.parent as u32) << 16) | (self.component as u32);
        buf.put_string(&self.text);
        buf.put_u16(0);
        buf.put_u32_mid_be(hash);

        Frame {
            opcode: 1,
            prefix: Prefix::Short,
            payload: buf.freeze(),
        }
    }
}
