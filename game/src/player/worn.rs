use std::{
    future::Future,
    ops::{Index, IndexMut},
    pin::Pin,
};

use filesystem::definition::WearFlag;
pub use filesystem::definition::WearPos;
use macros::player_system;
use net::{InvEntry, InvType, Outbox, OutboxExt, UpdateInvFull};
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
pub struct WornSlots(pub [Option<Obj>; SIZE]);

impl Index<WearPos> for WornSlots {
    type Output = Option<Obj>;
    fn index(&self, slot: WearPos) -> &Self::Output {
        &self.0[slot as usize]
    }
}

impl IndexMut<WearPos> for WornSlots {
    fn index_mut(&mut self, slot: WearPos) -> &mut Self::Output {
        &mut self.0[slot as usize]
    }
}

impl<T: AsRef<[Option<(u16, u32)>]>> From<T> for WornSlots {
    fn from(src: T) -> Self {
        let mut slots = [None; SIZE];
        for (i, slot) in src.as_ref().iter().enumerate().take(SIZE) {
            slots[i] = slot.map(|(id, amount)| Obj::new(id, amount));
        }
        Self(slots)
    }
}

pub struct Worn {
    outbox: Outbox,
    slots: WornSlots,
}

impl Worn {
    pub fn slot(&self, slot: WearPos) -> Option<Obj> {
        self.slots[slot]
    }

    pub fn set(&mut self, slot: WearPos, obj: Option<Obj>) {
        self.slots[slot] = obj;
    }

    pub fn slots(&self) -> &WornSlots {
        &self.slots
    }

    pub fn displace(&self, slot: WearPos, flag: WearFlag) -> Vec<Obj> {
        let occupant = self.slots[slot];
        let shield_conflict = (flag == WearFlag::TwoHanded)
            .then(|| self.slots[WearPos::Shield])
            .flatten();

        let weapon_conflict = (slot == WearPos::Shield)
            .then(|| self.slots[WearPos::Weapon])
            .flatten()
            .filter(|wep| {
                crate::provider::get_obj_type(wep.id as u32).is_some_and(|d| d.wearflag == WearFlag::TwoHanded)
            });

        [occupant, shield_conflict, weapon_conflict]
            .into_iter()
            .flatten()
            .collect()
    }

    pub async fn flush(&mut self) {
        self.outbox
            .write(UpdateInvFull {
                inv_type: InvType::Worn,
                negative_key: false,
                items: self
                    .slots
                    .0
                    .iter()
                    .map(|s| {
                        s.map(|obj| InvEntry {
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
impl PlayerSystem for Worn {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            outbox: ctx.outbox.clone(),
            slots: WornSlots::from(&ctx.player_data.worn),
        }
    }

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.flush().await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.worn = self.slots.0.iter().map(|s| s.map(|obj| (obj.id, obj.amount))).collect();
    }
}
