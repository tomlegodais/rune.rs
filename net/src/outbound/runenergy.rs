use tokio_util::bytes::{BufMut, BytesMut};

use crate::{Encodable, Frame, Prefix};

pub struct UpdateRunEnergy(pub u8);

impl Encodable for UpdateRunEnergy {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8(self.0);

        Frame {
            opcode: 18,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
