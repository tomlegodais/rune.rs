use util::BytesMutExt;

use crate::{Encodable, Frame, message::frame::FrameBuilder, outbound::zone::ZoneFrame};

pub struct LocAddChange {
    pub zone_frame: ZoneFrame,
    pub loc_id: u16,
    pub loc_type: u8,
    pub rotation: u8,
    pub packed_offset: u8,
}

pub struct LocDel {
    pub zone_frame: ZoneFrame,
    pub loc_type: u8,
    pub rotation: u8,
    pub packed_offset: u8,
}

impl Encodable for LocAddChange {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(70, |buf| {
            buf.put_u8_sub((self.loc_type << 2) | (self.rotation & 0x3));
            buf.put_u16_add(self.loc_id);
            buf.put_u8_add(self.packed_offset);
        })
    }
}

impl Encodable for LocDel {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(45, |buf| {
            buf.put_u8_add((self.loc_type << 2) | (self.rotation & 0x3));
            buf.put_u8_sub(self.packed_offset);
        })
    }
}
