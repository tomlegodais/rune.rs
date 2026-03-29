use std::{
    future::Future,
    ops::{Index, IndexMut},
    pin::Pin,
};

pub use filesystem::definition::EquipmentSlot;
use macros::player_system;
use net::{ItemContainerEntry, ItemContainerId, Outbox, OutboxExt, UpdateItemContainer};
use persistence::player::PlayerData;

use crate::{
    player::{
        PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
    },
    world::World,
};

pub const SIZE: usize = 14;

#[derive(Clone, Copy)]
pub struct EquipSlots(pub [Option<(u16, u32)>; SIZE]);

impl Index<EquipmentSlot> for EquipSlots {
    type Output = Option<(u16, u32)>;
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
            slots[i] = *slot;
        }
        Self(slots)
    }
}

pub struct Equipment {
    outbox: Outbox,
    slots: EquipSlots,
}

impl Equipment {
    pub fn slot(&self, slot: EquipmentSlot) -> Option<(u16, u32)> {
        self.slots[slot]
    }

    pub fn set(&mut self, slot: EquipmentSlot, item: Option<(u16, u32)>) {
        self.slots[slot] = item;
    }

    pub fn slots(&self) -> &EquipSlots {
        &self.slots
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
                    .map(|s| s.map(|(item_id, amount)| ItemContainerEntry { item_id, amount }))
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
        data.equipment = self.slots.0.to_vec();
    }
}
