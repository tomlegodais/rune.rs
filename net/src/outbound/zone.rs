use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct ZoneFrame {
    pub zone_x: u8,
    pub zone_y: u8,
    pub plane: u8,
}

impl ZoneFrame {
    pub fn new(zone_x: u8, zone_y: u8, plane: u8) -> Self {
        Self { zone_x, zone_y, plane }
    }
}

impl Encodable for ZoneFrame {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8_sub(self.zone_x);
        buf.put_u8(self.zone_y);
        buf.put_u8_neg(self.plane);

        Frame {
            opcode: 28,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
