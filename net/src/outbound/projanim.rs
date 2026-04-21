use tokio_util::bytes::BufMut;

use crate::{Encodable, Frame, message::frame::FrameBuilder, outbound::zone::ZoneFrame};

pub struct MapProjAnim {
    pub zone_frame: ZoneFrame,
    pub packed_pos: u8,
    pub dst_dx: i8,
    pub dst_dy: i8,
    pub target: i16,
    pub spotanim: u16,
    pub start_height: u8,
    pub end_height: u8,
    pub start_cycle: u16,
    pub end_cycle: u16,
    pub slope: u8,
    pub angle: u8,
}

impl Encodable for MapProjAnim {
    fn encode(self) -> Frame {
        FrameBuilder::embed(self.zone_frame).inner(57, |buf| {
            buf.put_u8(self.packed_pos);
            buf.put_i8(self.dst_dx);
            buf.put_i8(self.dst_dy);
            buf.put_i16(self.target);
            buf.put_u16(self.spotanim);
            buf.put_u8(self.start_height);
            buf.put_u8(self.end_height);
            buf.put_u16(self.start_cycle);
            buf.put_u16(self.end_cycle);
            buf.put_u8(self.slope);
            buf.put_u8(self.angle);
        })
    }
}
