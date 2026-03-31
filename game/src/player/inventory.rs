use std::{future::Future, pin::Pin};

use macros::player_system;
use net::{ItemContainerEntry, ItemContainerId, Outbox, OutboxExt, UpdateItemContainer, if_events, if_set_events};
use persistence::player::PlayerData;

use crate::{
    player::{
        Obj, PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
    },
    provider,
    world::World,
};

pub const STACK_MAX: u32 = i32::MAX as u32;
pub const SIZE: usize = 28;

pub struct Inventory {
    outbox: Outbox,
    slots: [Option<Obj>; SIZE],
}

impl Inventory {
    fn from_slots(outbox: Outbox, slots: [Option<Obj>; SIZE]) -> Self {
        Self { outbox, slots }
    }

    pub fn slot(&self, index: usize) -> Option<Obj> {
        self.slots[index]
    }

    pub fn count(&self, obj_id: u16) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|obj| obj.id == obj_id)
            .map(|obj| obj.amount)
            .sum()
    }

    pub async fn add(&mut self, obj_id: u16, amount: u32) -> u32 {
        let remaining = match is_stackable(obj_id) {
            true => self.add_stackable(obj_id, amount),
            false => self.add_unstackable(obj_id, amount),
        };

        self.flush().await;
        remaining
    }

    fn add_stackable(&mut self, obj_id: u16, amount: u32) -> u32 {
        if let Some(obj) = self.slots.iter_mut().flatten().find(|obj| obj.id == obj_id) {
            let added = amount.min(STACK_MAX - obj.amount);
            obj.amount += added;
            return amount - added;
        }

        match self.slots.iter_mut().find(|s| s.is_none()) {
            Some(slot) => {
                *slot = Some(Obj::new(obj_id, amount.min(STACK_MAX)));
                amount.saturating_sub(STACK_MAX)
            }
            None => amount,
        }
    }

    fn add_unstackable(&mut self, obj_id: u16, amount: u32) -> u32 {
        let free = self.slots.iter().filter(|s| s.is_none()).count() as u32;
        let added = amount.min(free);
        self.slots
            .iter_mut()
            .filter(|s| s.is_none())
            .take(added as usize)
            .for_each(|s| *s = Some(Obj::new(obj_id, 1)));

        amount - added
    }

    pub async fn remove(&mut self, obj_id: u16, amount: u32) -> u32 {
        let mut remaining = amount;
        for slot in self
            .slots
            .iter_mut()
            .filter(|s| s.map(|o| o.id == obj_id).unwrap_or(false))
        {
            let obj = slot.as_mut().expect("filtered to Some above");
            let taken = remaining.min(obj.amount);

            obj.amount -= taken;
            remaining -= taken;

            if obj.amount == 0 {
                *slot = None;
            }

            if remaining == 0 {
                break;
            }
        }
        self.flush().await;
        remaining
    }

    pub async fn set(&mut self, index: usize, obj: Option<Obj>) {
        self.slots[index] = obj;
        self.flush().await;
    }

    pub async fn clear(&mut self) {
        self.slots = [None; SIZE];
        self.flush().await;
    }

    pub async fn swap(&mut self, a: usize, b: usize) {
        self.slots.swap(a, b);
        self.flush().await;
    }

    pub fn free_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_none()).count()
    }

    pub async fn clear_slot(&mut self, index: usize) {
        self.slots[index] = None;
        self.flush().await;
    }

    pub async fn remove_obj(&mut self, slot: usize, amount: u32) {
        let Some(obj) = self.slots[slot] else { return };
        if !is_stackable(obj.id) || amount >= obj.amount {
            self.slots[slot] = None;
        } else {
            self.slots[slot] = Some(Obj::new(obj.id, obj.amount - amount));
        }
        self.flush().await;
    }

    async fn send_ifevents(&mut self) {
        self.outbox
            .write(if_set_events!(
                interface_id: 149,
                component_id: 0,
                slots: [0 => 27],
                right_click[0,1,2,6,7,8],
                use_on[ground, npcs, locs, components],
                depth[1],
                can_use_on
            ))
            .await;

        self.outbox
            .write(if_set_events!(
                interface_id: 149, component_id: 0, slots: [28 => 55], can_drag_onto
            ))
            .await;
    }

    pub async fn flush(&mut self) {
        self.outbox
            .write(UpdateItemContainer {
                container: ItemContainerId::Inventory,
                negative_key: false,
                items: self
                    .slots
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

fn is_stackable(obj_id: u16) -> bool {
    provider::get_item_definition(obj_id as u32).is_some_and(|def| def.stackable)
}

#[player_system]
impl PlayerSystem for Inventory {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        let mut slots = [None; SIZE];
        for (i, slot) in ctx.player_data.inventory.iter().enumerate().take(SIZE) {
            slots[i] = slot.map(|(id, amount)| Obj::new(id, amount));
        }
        Self::from_slots(ctx.outbox.clone(), slots)
    }

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.flush().await;
            self.send_ifevents().await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.inventory = self
            .slots
            .iter()
            .map(|s| s.map(|obj| (obj.id, obj.amount)))
            .collect();
    }
}
