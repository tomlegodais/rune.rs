use tokio_util::bytes::{Buf, BufMut};

pub trait BufExt: Buf {
    fn get_u24(&mut self) -> u32 {
        (self.get_u8() as u32) << 16 | (self.get_u8() as u32) << 8 | self.get_u8() as u32
    }

    fn get_smart_u32(&mut self) -> u32 {
        let peek = self.chunk()[0];
        if peek & 0x80 == 0 {
            self.get_u16() as u32
        } else {
            let value = self.get_u32();
            value & 0x7FFFFFFF
        }
    }

    fn get_string(&mut self) -> String {
        let mut bytes = Vec::new();

        while self.has_remaining() {
            let b = self.get_u8();
            if b == 0 {
                break;
            }
            bytes.push(b);
        }

        String::from_utf8_lossy(&bytes).into_owned()
    }
}

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

impl<T: Buf> BufExt for T {}
impl<T: BufMut> BytesMutExt for T {}
