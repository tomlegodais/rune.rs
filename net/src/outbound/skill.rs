use crate::{Encodable, Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

pub struct UpdateSkill {
    pub id: u8,
    pub level: u8,
    pub xp: u32,
}

impl Encodable for UpdateSkill {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u32_le(self.xp);
        buf.put_u8_add(self.id);
        buf.put_u8_sub(self.level);

        Frame {
            opcode: 9,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
