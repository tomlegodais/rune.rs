use tokio_util::bytes::{BufMut, BytesMut};

use crate::{Encodable, Frame, Prefix};

pub struct IfCloseSub {
    pub parent: u16,
    pub component: u16,
}

impl IfCloseSub {
    fn hash(&self) -> u32 {
        ((self.parent as u32) << 16) | (self.component as u32)
    }
}

impl Encodable for IfCloseSub {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16(0);
        buf.put_u32(self.hash());

        Frame {
            opcode: 44,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
