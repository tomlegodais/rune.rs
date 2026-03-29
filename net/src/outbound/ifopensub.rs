use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct IfOpenSub {
    pub parent: u16,
    pub component: u16,
    pub interface: u16,
    pub transparent: bool,
}

impl IfOpenSub {
    pub fn hash(&self) -> u32 {
        ((self.parent as u32) << 16) | (self.component as u32)
    }
}

impl Encodable for IfOpenSub {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8_sub(self.transparent as u8);
        buf.put_u16_add(0);
        buf.put_u32(self.hash());
        buf.put_u16_le(self.interface);

        Frame {
            opcode: 22,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
