use tokio_util::bytes::{BufMut, BytesMut};

use crate::{Encodable, Frame, Prefix};

pub struct IfOpenTop {
    pub interface: u16,
    pub sub: u8,
}

impl IfOpenTop {
    pub fn new(interface: u16) -> Self {
        Self { interface, sub: 0 }
    }
}

impl Encodable for IfOpenTop {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8(self.sub);
        buf.put_u16_le(0);
        buf.put_u16_le(self.interface);

        Frame {
            opcode: 102,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
