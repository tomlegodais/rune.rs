use crate::{Encodable, Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};

pub struct IfOpenTop(pub u16);

impl Encodable for IfOpenTop {
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
