use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct IfSetNpcHead {
    pub interface_id: u16,
    pub component: u16,
    pub npc_id: u16,
}

impl Encodable for IfSetNpcHead {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        let hash = ((self.interface_id as u32) << 16) | (self.component as u32);
        buf.put_u32_le(hash);
        buf.put_u16(0);
        buf.put_u16_add(self.npc_id);

        Frame {
            opcode: 39,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
