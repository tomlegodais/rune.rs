use tokio_util::bytes::BytesMut;
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub struct SetPlayerOp {
    pub slot: u8,
    pub top: bool,
    pub op: String,
}

impl Encodable for SetPlayerOp {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u8_sub(self.slot);
        buf.put_u8_neg(if self.top { 1 } else { 0 });
        buf.put_u16_add(if self.slot == 1 && self.top { 0x2A } else { 0xFFFF });
        buf.put_string(&self.op);

        Frame {
            opcode: 74,
            prefix: Prefix::Byte,
            payload: buf.freeze(),
        }
    }
}
