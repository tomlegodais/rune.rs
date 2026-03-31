use std::collections::HashMap;

use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

#[derive(Debug, Clone, PartialEq)]
pub enum EnumValue {
    Int(i32),
    String(String),
}

#[derive(Debug, Clone)]
pub struct EnumType {
    pub id: u32,
    pub key_type: char,
    pub value_type: char,
    pub default_string: String,
    pub default_int: i32,
    pub values: HashMap<i32, EnumValue>,
}

impl Default for EnumType {
    fn default() -> Self {
        Self {
            id: 0,
            key_type: '\0',
            value_type: '\0',
            default_string: String::new(),
            default_int: 0,
            values: HashMap::new(),
        }
    }
}

impl EnumType {
    pub fn decode(id: u32, data: &[u8]) -> anyhow::Result<Self> {
        let mut def = Self {
            id,
            ..Default::default()
        };
        let mut buf = Bytes::copy_from_slice(data);
        loop {
            let opcode = buf.get_u8();
            if opcode == 0 {
                break;
            }
            def.decode_opcode(&mut buf, opcode)?;
        }
        Ok(def)
    }

    fn decode_opcode(&mut self, buf: &mut Bytes, opcode: u8) -> anyhow::Result<()> {
        match opcode {
            1 => self.key_type = cp1252_to_char(buf.get_u8()),
            2 => self.value_type = cp1252_to_char(buf.get_u8()),
            3 => self.default_string = buf.get_string(),
            4 => self.default_int = buf.get_i32(),
            5 | 6 => {
                let count = buf.get_u16() as usize;
                self.values = HashMap::with_capacity(count);
                for _ in 0..count {
                    let key = buf.get_i32();
                    let value =
                        if opcode == 5 { EnumValue::String(buf.get_string()) } else { EnumValue::Int(buf.get_i32()) };
                    self.values.insert(key, value);
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn int_value(&self, key: i32) -> Option<i32> {
        match self.values.get(&key)? {
            EnumValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn str_value(&self, key: i32) -> Option<&str> {
        match self.values.get(&key)? {
            EnumValue::String(v) => Some(v),
            _ => None,
        }
    }
}

fn cp1252_to_char(byte: u8) -> char {
    const TABLE: [char; 32] = [
        '\u{20ac}', '\0', '\u{201a}', '\u{0192}', '\u{201e}', '\u{2026}', '\u{2020}', '\u{2021}', '\u{02c6}',
        '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\0', '\u{017d}', '\0', '\0', '\u{2018}', '\u{2019}',
        '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{02dc}', '\u{2122}', '\u{0161}', '\u{203a}',
        '\u{0153}', '\0', '\u{017e}', '\u{0178}',
    ];
    if byte == 0 {
        return '\0';
    }
    if (0x80..0xa0).contains(&byte) {
        let c = TABLE[(byte - 0x80) as usize];
        return if c == '\0' { '?' } else { c };
    }
    byte as char
}
