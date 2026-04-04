use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct VarcSmall {
    pub id: u16,
    pub value: u8,
}

pub struct VarcLarge {
    pub id: u16,
    pub value: u32,
}

impl Encodable for VarcSmall {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16_le_add(0);
        buf.put_u16(self.id);
        buf.put_u8_add(self.value);

        Frame {
            opcode: 12,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}

impl Encodable for VarcLarge {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16_add(0);
        buf.put_u16_le(self.id);
        buf.put_u32_le(self.value);

        Frame {
            opcode: 52,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
