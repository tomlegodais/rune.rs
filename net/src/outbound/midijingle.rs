use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct MidiJingle {
    pub id: u16,
    pub delay: u32,
    pub volume: u8,
}

impl Encodable for MidiJingle {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u24(self.delay);
        buf.put_u16_le_add(self.id);
        buf.put_u8(self.volume);

        Frame {
            opcode: 21,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
