use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvType {
    Inv,
    Worn,
    Bank,
    Custom(u16),
}

impl InvType {
    pub const fn key(self) -> u16 {
        match self {
            InvType::Inv => 93,
            InvType::Worn => 94,
            InvType::Bank => 95,
            InvType::Custom(key) => key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvEntry {
    pub item_id: u16,
    pub amount: u32,
}

pub struct UpdateInvFull {
    pub inv_type: InvType,
    pub negative_key: bool,
    pub items: Vec<Option<InvEntry>>,
}

impl Encodable for UpdateInvFull {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();

        buf.put_u16(self.inv_type.key());
        buf.put_u8(self.negative_key as u8);
        buf.put_u16(self.items.len() as u16);

        for slot in self.items {
            match slot {
                Some(entry) => {
                    if entry.amount < 255 {
                        buf.put_u8_sub(entry.amount as u8);
                    } else {
                        buf.put_u8_sub(255);
                        buf.put_u32_le(entry.amount);
                    }
                    buf.put_u16(entry.item_id.saturating_add(1));
                }
                None => {
                    buf.put_u8_sub(0);
                    buf.put_u16(0);
                }
            }
        }

        Frame {
            opcode: 11,
            prefix: Prefix::Short,
            payload: buf.freeze(),
        }
    }
}
