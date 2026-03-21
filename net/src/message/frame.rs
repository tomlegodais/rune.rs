use tokio_util::bytes::Bytes;

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

pub trait Encodable {
    fn encode(self) -> Frame;
}
