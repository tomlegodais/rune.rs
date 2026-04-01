use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct IfSetPlayerHead {
    pub interface_id: u16,
    pub component: u16,
}

impl Encodable for IfSetPlayerHead {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        let hash = ((self.interface_id as u32) << 16) | (self.component as u32);
        buf.put_u32_mid_be(hash);
        buf.put_u16(0);

        Frame {
            opcode: 24,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
