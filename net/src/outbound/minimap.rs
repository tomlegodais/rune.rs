use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct MinimapToggle {
    pub x: u8,
    pub y: u8,
}

impl MinimapToggle {
    pub fn reset() -> Self {
        Self { x: 255, y: 255 }
    }
}

impl Encodable for MinimapToggle {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8(self.x);
        buf.put_u8_neg(self.y);

        Frame {
            opcode: 84,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
