use tokio_util::bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Copy, Clone)]
pub enum Prefix {
    Fixed,
    Byte,
    Short,
}

#[derive(Debug)]
pub struct Frame {
    pub opcode: u8,
    pub prefix: Prefix,
    pub payload: Bytes,
}

pub struct FrameBuilder {
    opcode: u8,
    buf: BytesMut,
}

impl FrameBuilder {
    pub fn embed(parent: impl Encodable) -> Self {
        let Frame { opcode, payload, .. } = parent.encode();

        let mut buf = BytesMut::with_capacity(payload.len() + 8);
        buf.extend_from_slice(&payload);

        Self { opcode, buf }
    }

    pub fn inner(mut self, opcode: u8, f: impl FnOnce(&mut BytesMut)) -> Frame {
        self.buf.put_u8(opcode);
        f(&mut self.buf);

        Frame {
            opcode: self.opcode,
            prefix: Prefix::Fixed,
            payload: self.buf.freeze(),
        }
    }
}

pub trait Encodable {
    fn encode(self) -> Frame;
}
