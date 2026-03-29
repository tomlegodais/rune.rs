use std::collections::HashMap;

use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use crate::definition::ParamValue;

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub id: u32,
    pub params: HashMap<u32, ParamValue>,
}

impl StructDefinition {
    pub fn decode(id: u32, data: &[u8]) -> anyhow::Result<Self> {
        let mut def = Self {
            id,
            params: HashMap::new(),
        };
        let mut buf = Bytes::copy_from_slice(data);
        loop {
            match buf.get_u8() {
                0 => break,
                249 => {
                    let count = buf.get_u8() as usize;
                    for _ in 0..count {
                        let is_string = buf.get_u8() == 1;
                        let key = buf.get_u24();
                        let value = if is_string {
                            ParamValue::String(buf.get_string())
                        } else {
                            ParamValue::Int(buf.get_i32())
                        };
                        def.params.insert(key, value);
                    }
                }
                _ => {}
            }
        }
        Ok(def)
    }
}
