use net::{Encodable, Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

pub struct SetRootWidget(pub u16);

impl Encodable for SetRootWidget {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8(0);
        buf.put_u16_le(0);
        buf.put_u16_le(self.0);

        Frame {
            opcode: 102,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}

pub struct OpenWidget {
    pub parent: u16,
    pub position: u16,
    pub interface: u16,
    pub click_through: bool,
}

impl Encodable for OpenWidget {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8_sub(self.click_through as u8);
        buf.put_u16_add(0);
        buf.put_u32(self.to_hash());
        buf.put_u16_le(self.interface);

        Frame {
            opcode: 22,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}

impl OpenWidget {
    pub fn to_hash(&self) -> u32 {
        ((self.parent as u32) << 16) | (self.position as u32)
    }
}