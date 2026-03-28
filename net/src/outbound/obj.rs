use crate::message::frame::FrameBuilder;
use crate::outbound::zone::ZoneFrame;
use crate::{Encodable, Frame};
use tokio_util::bytes::BufMut;
use util::BytesMutExt;

pub struct ObjAdd {
    pub zone_frame: ZoneFrame,
    pub item_id: u16,
    pub amount: u32,
    pub packed_offset: u8,
}

pub struct ObjDel {
    pub zone_frame: ZoneFrame,
    pub item_id: u16,
    pub packed_offset: u8,
}

impl Encodable for ObjAdd {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(68, |buf| {
            buf.put_u16_le(self.amount as u16);
            buf.put_u16_add(self.item_id);
            buf.put_u8(self.packed_offset);
        })
    }
}

impl Encodable for ObjDel {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(43, |buf| {
            buf.put_u16_le_add(self.item_id);
            buf.put_u8_neg(self.packed_offset);
        })
    }
}
