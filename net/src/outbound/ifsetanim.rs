use tokio_util::bytes::BytesMut;
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct IfSetAnim {
    pub interface_id: u16,
    pub component: u16,
    pub anim_id: u16,
}

impl Encodable for IfSetAnim {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        let hash = ((self.interface_id as u32) << 16) | (self.component as u32);
        buf.put_u16_add(0);
        buf.put_u16_add(self.anim_id);
        buf.put_u32_mid_be(hash);

        Frame {
            opcode: 86,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
