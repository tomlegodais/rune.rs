use crate::player::system::{PlayerInitContext, PlayerSystem, SystemContext};
use crate::provider;
use macros::player_system;
use net::{
    ChatMessage, ItemContainerEntry, ItemContainerId, Outbox, OutboxExt, UpdateItemContainer,
    if_events, if_set_events,
};
use persistence::player::PlayerData;
use std::future::Future;
use std::pin::Pin;

const STACK_MAX: u32 = i32::MAX as u32;
const SIZE: usize = 28;

pub struct Inventory {
    outbox: Outbox,
    slots: [Option<(u16, u32)>; SIZE],
}

impl Inventory {
    fn from_slots(outbox: Outbox, slots: [Option<(u16, u32)>; SIZE]) -> Self {
        Self { outbox, slots }
    }

    #[allow(dead_code)]
    pub fn slot(&self, index: usize) -> Option<(u16, u32)> {
        self.slots[index]
    }

    #[allow(dead_code)]
    pub fn count(&self, item_id: u16) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|(id, _)| *id == item_id)
            .map(|(_, qty)| *qty)
            .sum()
    }

    pub async fn add(&mut self, item_id: u16, amount: u32) -> u32 {
        let remaining = match is_stackable(item_id) {
            true => self.add_stackable(item_id, amount),
            false => self.add_unstackable(item_id, amount),
        };

        self.flush().await;

        if remaining > 0 {
            self.outbox
                .write(ChatMessage {
                    msg_type: 0,
                    text: "You can't carry any more items.".to_string(),
                })
                .await;
        }

        remaining
    }

    #[rustfmt::skip]
    fn add_stackable(&mut self, item_id: u16, amount: u32) -> u32 {
        if let Some((_, qty)) = self.slots.iter_mut().flatten().find(|(id, _)| *id == item_id) {
            let added = amount.min(STACK_MAX - *qty);
            *qty += added;
            return amount - added;
        }

        match self.slots.iter_mut().find(|s| s.is_none()) {
            Some(slot) => { *slot = Some((item_id, amount.min(STACK_MAX))); amount.saturating_sub(STACK_MAX) }
            None => amount,
        }
    }

    fn add_unstackable(&mut self, item_id: u16, amount: u32) -> u32 {
        let free = self.slots.iter().filter(|s| s.is_none()).count() as u32;
        let added = amount.min(free);

        self.slots
            .iter_mut()
            .filter(|s| s.is_none())
            .take(added as usize)
            .for_each(|s| *s = Some((item_id, 1)));

        amount - added
    }

    pub async fn remove(&mut self, item_id: u16, amount: u32) -> u32 {
        let mut remaining = amount;

        for slot in self
            .slots
            .iter_mut()
            .filter(|s| s.map(|(id, _)| id == item_id).unwrap_or(false))
        {
            let (_, qty) = slot.as_mut().unwrap();
            let taken = remaining.min(*qty);
            *qty -= taken;
            remaining -= taken;
            if *qty == 0 {
                *slot = None;
            }
            if remaining == 0 {
                break;
            }
        }

        self.flush().await;
        remaining
    }

    #[allow(dead_code)]
    pub async fn set(&mut self, index: usize, item: Option<(u16, u32)>) {
        self.slots[index] = item;
        self.flush().await;
    }

    pub async fn clear(&mut self) {
        self.slots = [None; SIZE];
        self.flush().await;
    }

    async fn send_ifevents(&mut self) {
        self.outbox
            .write(if_set_events!(
                interface_id: 149,
                component_id: 0,
                slots: [0 => 27],
                right_click[0,1,2,6,7,8],
                use_on[ground, npcs, objects, components],
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
                    .map(|s| s.map(|(item_id, amount)| ItemContainerEntry { item_id, amount }))
                    .collect(),
            })
            .await;
    }
}

fn is_stackable(item_id: u16) -> bool {
    provider::get_item_definition(item_id as u32).is_some_and(|def| def.stackable)
}

#[player_system]
impl PlayerSystem for Inventory {
    fn create(ctx: &PlayerInitContext) -> Self {
        let mut slots = [None; SIZE];
        for (i, slot) in ctx.data.inventory.iter().enumerate().take(SIZE) {
            slots[i] = *slot;
        }
        Self::from_slots(ctx.outbox.clone(), slots)
    }

    fn on_login<'a>(
        &'a mut self,
        _ctx: &'a mut SystemContext<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.flush().await;
            self.send_ifevents().await;
        })
    }

    fn persist(&self, data: &mut PlayerData) {
        data.inventory = self.slots.iter().copied().collect();
    }
}
