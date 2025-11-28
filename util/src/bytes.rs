use tokio_util::bytes::{BufMut, BytesMut};

pub trait BytesMutExt: BufMut {
    fn put_smart(&mut self, value: u16) {
        if value < 128 {
            self.put_u8(value as u8);
        } else {
            self.put_u8(((value >> 8) as u8) | 0x80);
            self.put_u8(value as u8);
        }
    }

    fn put_jag_string(&mut self, value: &str) {
        self.put_u8(0u8);
        self.put_slice(value.as_bytes());
        self.put_u8(0u8);
    }
}

impl BytesMutExt for BytesMut {}
impl BytesMutExt for &mut BytesMut {}
