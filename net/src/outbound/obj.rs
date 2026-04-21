use tokio_util::bytes::BufMut;
use util::BytesMutExt;

use crate::{Encodable, Frame, message::frame::FrameBuilder, outbound::zone::ZoneFrame};

pub struct ObjAdd {
    pub zone_frame: ZoneFrame,
    pub obj_id: u16,
    pub amount: u32,
    pub packed_offset: u8,
}

pub struct ObjDel {
    pub zone_frame: ZoneFrame,
    pub obj_id: u16,
    pub packed_offset: u8,
}

pub struct ObjCount {
    pub zone_frame: ZoneFrame,
    pub obj_id: u16,
    pub old_amount: u32,
    pub new_amount: u32,
    pub packed_offset: u8,
}

impl Encodable for ObjAdd {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(68, |buf| {
            buf.put_u16_le(self.amount as u16);
            buf.put_u16_add(self.obj_id);
            buf.put_u8(self.packed_offset);
        })
    }
}

impl Encodable for ObjDel {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(43, |buf| {
            buf.put_u16_le_add(self.obj_id);
            buf.put_u8_neg(self.packed_offset);
        })
    }
}

impl Encodable for ObjCount {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(80, |buf| {
            buf.put_u8(self.packed_offset);
            buf.put_u16(self.obj_id);
            buf.put_u16(self.old_amount as u16);
            buf.put_u16(self.new_amount as u16);
        })
    }
}
