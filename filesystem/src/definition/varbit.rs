use std::io;

use tokio_util::bytes::{Buf, Bytes};

#[derive(Debug, Clone)]
pub struct VarbitType {
    pub id: u32,
    pub varp: u16,
    pub low_bit: u8,
    pub high_bit: u8,
}

impl VarbitType {
    pub fn decode(id: u32, data: &[u8]) -> io::Result<Self> {
        let mut buf = Bytes::copy_from_slice(data);
        let mut t = Self {
            id,
            varp: 0,
            low_bit: 0,
            high_bit: 0,
        };

        loop {
            match buf.get_u8() {
                0 => break,
                1 => {
                    t.varp = buf.get_u16();
                    t.low_bit = buf.get_u8();
                    t.high_bit = buf.get_u8();
                }
                _ => {}
            }
        }

        Ok(t)
    }

    pub fn mask(&self) -> u32 {
        (1 << (self.high_bit - self.low_bit + 1)) - 1
    }
}
