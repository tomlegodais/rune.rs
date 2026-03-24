use crate::world::Varbits;
use net::{LargeVarbit, LargeVarp, Outbox, OutboxExt, SmallVarbit, SmallVarp};
use std::collections::HashMap;

pub struct VarpManager {
    outbox: Outbox,
    varps: HashMap<u16, i32>,
}

impl VarpManager {
    pub fn new(outbox: Outbox) -> Self {
        Self {
            outbox,
            varps: HashMap::new(),
        }
    }

    pub async fn on_login(&mut self) {
        self.send_varp(281, 1000).await;
        self.send_varp(1160, -1).await;
        self.send_varp(1159, 1).await;
    }

    pub fn get(&self, id: u16) -> i32 {
        self.varps.get(&id).copied().unwrap_or(0)
    }

    pub async fn send_varp(&mut self, id: u16, value: i32) {
        self.varps.insert(id, value);
        if value >= i8::MIN as i32 && value <= i8::MAX as i32 {
            self.outbox
                .write(SmallVarp {
                    id,
                    value: value as u8,
                })
                .await;
        } else {
            self.outbox
                .write(LargeVarp {
                    id,
                    value: value as u32,
                })
                .await;
        }
    }

    pub async fn send_varbit(&mut self, id: u32, value: i32) {
        let Some(def) = Varbits::get(id) else { return };

        let mask = def.mask() as i32;
        let current = self.get(def.varp);
        let updated = (current & !(mask << def.low_bit)) | ((value & mask) << def.low_bit);

        self.send_varp(def.varp, updated).await;

        if value <= u8::MAX as i32 {
            self.outbox
                .write(SmallVarbit {
                    id: id as u16,
                    value: value as u8,
                })
                .await;
        } else {
            self.outbox
                .write(LargeVarbit {
                    id: id as u16,
                    value: value as u32,
                })
                .await;
        }
    }
}
