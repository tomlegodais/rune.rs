use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

pub enum ScriptArg {
    Int(i32),
    Str(String),
}

pub struct RunClientScript {
    pub id: u32,
    pub args: Vec<ScriptArg>,
}

impl Encodable for RunClientScript {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();

        buf.put_u16(0);

        let type_mask: String = self
            .args
            .iter()
            .rev()
            .map(|a| match a {
                ScriptArg::Int(_) => 'i',
                ScriptArg::Str(_) => 's',
            })
            .collect();

        buf.put_string(&type_mask);

        for arg in &self.args {
            match arg {
                ScriptArg::Int(v) => buf.put_i32(*v),
                ScriptArg::Str(s) => buf.put_string(s),
            }
        }

        buf.put_u32(self.id);

        Frame {
            opcode: 27,
            prefix: Prefix::Short,
            payload: buf.freeze(),
        }
    }
}
