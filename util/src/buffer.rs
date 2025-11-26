use std::io;
use std::io::ErrorKind;

#[derive(Debug)]
pub struct Buffer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.pos < self.data.len()
    }

    pub fn peek_u8(&self) -> io::Result<u8> {
        self.data
            .get(self.pos)
            .copied()
            .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"))
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        let value = self.peek_u8()?;
        self.pos += 1;
        Ok(value)
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        if self.pos + 2 > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"));
        }
        let value = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(value)
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    pub fn read_u24(&mut self) -> io::Result<u32> {
        if self.pos + 3 > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"));
        }
        let value = u32::from_be_bytes([
            0,
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
        ]);
        self.pos += 3;
        Ok(value)
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        if self.pos + 4 > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"));
        }
        let value = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(value)
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_smart(&mut self) -> io::Result<u16> {
        let peek = self.peek_u8()?;
        if peek < 128 {
            Ok(self.read_u8()? as u16)
        } else {
            Ok(self.read_u16()? - 32768)
        }
    }

    pub fn read_smart_signed(&mut self) -> io::Result<i16> {
        let peek = self.peek_u8()?;
        if peek < 128 {
            Ok(self.read_u8()? as i16 - 64)
        } else {
            Ok((self.read_u16()? as i32 - 49152) as i16)
        }
    }

    pub fn read_smart_u32(&mut self) -> io::Result<u32> {
        if self.pos >= self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"));
        }

        if self.data[self.pos] & 0x80 == 0 {
            Ok(self.read_u16()? as u32)
        } else {
            let value = self.read_u32()?;
            Ok(value & 0x7FFFFFFF)
        }
    }

    pub fn read_big_smart(&mut self) -> io::Result<i32> {
        let peek = self.peek_u8()?;
        if peek < 128 {
            Ok(self.read_u16()? as i32 - 1)
        } else {
            Ok(self.read_u32()? as i32 - 1)
        }
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }

        let s = String::from_utf8_lossy(&self.data[start..self.pos]).into_owned();
        if self.pos < self.data.len() {
            self.pos += 1;
        }

        Ok(s)
    }

    pub fn read_versioned_string(&mut self) -> io::Result<String> {
        let _version = self.read_u8()?;
        self.read_string()
    }

    pub fn skip(&mut self, count: usize) -> io::Result<()> {
        if self.pos + count > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"));
        }
        self.pos += count;
        Ok(())
    }

    pub fn read_bytes(&mut self, count: usize) -> io::Result<&'a [u8]> {
        if self.pos + count > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "buffer underflow"));
        }
        let slice = &self.data[self.pos..self.pos + count];
        self.pos += count;
        Ok(slice)
    }

    pub fn remaining_bytes(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }
}
