use std::{
    future::Future,
    ops::{Index, IndexMut},
    pin::Pin,
};

use filesystem::definition::EquipmentFlag;
pub use filesystem::definition::EquipmentSlot;
use macros::player_system;
use net::{ItemContainerEntry, ItemContainerId, Outbox, OutboxExt, UpdateItemContainer};
use persistence::player::PlayerData;

use crate::{
    player::{
        Obj, PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
    },
    world::World,
};

pub const SIZE: usize = 14;

#[derive(Clone, Copy)]
pub struct EquipSlots(pub [Option<Obj>; SIZE]);

impl Index<EquipmentSlot> for EquipSlots {
    type Output = Option<Obj>;
    fn index(&self, slot: EquipmentSlot) -> &Self::Output {
        &self.0[slot as usize]
    }
}

impl IndexMut<EquipmentSlot> for EquipSlots {
    fn index_mut(&mut self, slot: EquipmentSlot) -> &mut Self::Output {
        &mut self.0[slot as usize]
    }
}

impl<T: AsRef<[Option<(u16, u32)>]>> From<T> for EquipSlots {
    fn from(src: T) -> Self {
        let mut slots = [None; SIZE];
        for (i, slot) in src.as_ref().iter().enumerate().take(SIZE) {
            slots[i] = slot.map(|(id, amount)| Obj::new(id, amount));
        }
        Self(slots)
    }
}

pub struct Equipment {
    outbox: Outbox,
    slots: EquipSlots,
}

impl Equipment {
    pub fn slot(&self, slot: EquipmentSlot) -> Option<Obj> {
        self.slots[slot]
    }

    pub fn set(&mut self, slot: EquipmentSlot, obj: Option<Obj>) {
        self.slots[slot] = obj;
    }

    pub fn slots(&self) -> &EquipSlots {
        &self.slots
    }

    pub fn displace(&self, slot: EquipmentSlot, flag: EquipmentFlag) -> Vec<Obj> {
        let occupant = self.slots[slot];
        let shield_conflict = (flag == EquipmentFlag::TwoHanded)
            .then(|| self.slots[EquipmentSlot::Shield])
            .flatten();

        let weapon_conflict = (slot == EquipmentSlot::Shield)
            .then(|| self.slots[EquipmentSlot::Weapon])
            .flatten()
            .filter(|wep| {
                crate::provider::get_obj_type(wep.id as u32)
                    .is_some_and(|d| d.equipment_flag == EquipmentFlag::TwoHanded)
            });

        [occupant, shield_conflict, weapon_conflict]
            .into_iter()
            .flatten()
            .collect()
    }

    pub async fn flush(&mut self) {
        self.outbox
            .write(UpdateItemContainer {
                container: ItemContainerId::Equipment,
                negative_key: false,
                items: self
                    .slots
                    .0
                    .iter()
                    .map(|s| {
                        s.map(|obj| ItemContainerEntry {
                            item_id: obj.id,
                            amount: obj.amount,
                        })
                    })
                    .collect(),
            })
            .await;
    }
}

#[player_system]
impl PlayerSystem for Equipment {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            outbox: ctx.outbox.clone(),
            slots: EquipSlots::from(&ctx.player_data.equipment),
        }
    }

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.flush().await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.equipment = self
            .slots
            .0
            .iter()
            .map(|s| s.map(|obj| (obj.id, obj.amount)))
            .collect();
    }
}
