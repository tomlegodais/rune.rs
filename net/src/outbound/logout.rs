use crate::{Encodable, Frame, Prefix};
use tokio_util::bytes::BytesMut;

pub struct Logout;

impl Encodable for Logout {
    fn encode(self) -> Frame {
        Frame {
            opcode: 33,
            prefix: Prefix::Fixed,
            payload: BytesMut::new().freeze(),
        }
    }
}
