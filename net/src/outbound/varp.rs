use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct SmallVarp {
    pub id: u16,
    pub value: u8,
}

pub struct LargeVarp {
    pub id: u16,
    pub value: u32,
}

pub struct SmallVarbit {
    pub id: u16,
    pub value: u8,
}

pub struct LargeVarbit {
    pub id: u16,
    pub value: u32,
}

impl Encodable for SmallVarp {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16(self.id);
        buf.put_u8_neg(self.value);

        Frame {
            opcode: 40,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}

impl Encodable for LargeVarp {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u32_mid_le(self.value);
        buf.put_u16_le(self.id);

        Frame {
            opcode: 79,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}

impl Encodable for SmallVarbit {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16(self.id);
        buf.put_u8(self.value);

        Frame {
            opcode: 14,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}

impl Encodable for LargeVarbit {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16_le_add(self.id);
        buf.put_u32_le(self.value);

        Frame {
            opcode: 85,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
