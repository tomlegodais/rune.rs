use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{Encodable, Frame, Prefix};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemContainerId {
    Inventory,
    Equipment,
    Bank,
    Custom(u16),
}

impl ItemContainerId {
    pub const fn key(self) -> u16 {
        match self {
            ItemContainerId::Inventory => 93,
            ItemContainerId::Equipment => 94,
            ItemContainerId::Bank => 95,
            ItemContainerId::Custom(key) => key,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemContainerEntry {
    pub item_id: u16,
    pub amount: u32,
}

pub struct UpdateItemContainer {
    pub container: ItemContainerId,
    pub negative_key: bool,
    pub items: Vec<Option<ItemContainerEntry>>,
}

impl Encodable for UpdateItemContainer {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();

        buf.put_u16(self.container.key());
        buf.put_u8(self.negative_key as u8);
        buf.put_u16(self.items.len() as u16);

        for slot in self.items {
            match slot {
                Some(item) => {
                    if item.amount < 255 {
                        buf.put_u8_sub(item.amount as u8);
                    } else {
                        buf.put_u8_sub(255);
                        buf.put_u32_le(item.amount);
                    }
                    buf.put_u16(item.item_id.saturating_add(1));
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
