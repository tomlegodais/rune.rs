use bytes::BytesMut;
use tokio_util::bytes::{Buf, BufMut};

const NULL_TERMINATOR: u8 = 0;

pub trait BitsMut {
    fn bits_start(&self) -> usize;
    fn put_bits(&mut self, bit_pos: &mut usize, num_bits: usize, value: u32);
    fn bits_end(&mut self, bit_pos: usize);
}

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
            if b == NULL_TERMINATOR {
                break;
            }
            bytes.push(b);
        }

        String::from_utf8_lossy(&bytes).into_owned()
    }
}

pub trait BytesMutExt: BufMut {
    fn put_u8_sub(&mut self, value: u8) {
        self.put_u8(128u8.wrapping_sub(value));
    }

    fn put_u8_add(&mut self, value: u8) {
        self.put_u8(value.wrapping_add(128));
    }

    fn put_u16_add(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.put_u8(hi);
        self.put_u8(lo.wrapping_add(128));
    }

    fn put_u16_le_add(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.put_u8(lo.wrapping_add(128));
        self.put_u8(hi);
    }

    fn put_smart(&mut self, value: u16) {
        if value < 128 {
            self.put_u8(value as u8);
        } else {
            self.put_u8(((value >> 8) as u8) | 0x80);
            self.put_u8(value as u8);
        }
    }

    fn put_versioned_string(&mut self, version: u8, value: &str) {
        self.put_u8(version);
        self.put_slice(value.as_bytes());
        self.put_u8(NULL_TERMINATOR);
    }
}

impl BitsMut for BytesMut {
    fn bits_start(&self) -> usize {
        self.len() * 8
    }

    fn put_bits(&mut self, bit_pos: &mut usize, mut num_bits: usize, value: u32) {
        let mut byte_pos = *bit_pos >> 3;
        let mut bit_offset = 8 - (*bit_pos & 7);
        *bit_pos += num_bits;

        let ensure_byte = |buf: &mut BytesMut, idx: usize| {
            if buf.len() <= idx {
                buf.resize(idx + 1, 0);
            }
        };

        while num_bits > bit_offset {
            ensure_byte(self, byte_pos);
            let mask = (1u32 << bit_offset) - 1;
            let to_write = (value >> (num_bits - bit_offset)) & mask;

            self[byte_pos] &= !(mask as u8);
            self[byte_pos] |= to_write as u8;

            byte_pos += 1;
            num_bits -= bit_offset;
            bit_offset = 8;
        }

        ensure_byte(self, byte_pos);
        let mask = (1u32 << bit_offset) - 1;

        if num_bits == bit_offset {
            self[byte_pos] &= !(mask as u8);
            self[byte_pos] |= (value & mask) as u8;
        } else {
            self[byte_pos] &= !((mask as u8) << (bit_offset - num_bits));
            self[byte_pos] |= ((value & mask) as u8) << (bit_offset - num_bits);
        }
    }

    fn bits_end(&mut self, bit_pos: usize) {
        let pos = (bit_pos + 7) / 8;
        if self.len() < pos {
            self.resize(pos, 0);
        }
    }
}

impl<T: Buf> BufExt for T {}

impl<T: BufMut> BytesMutExt for T {}
