use std::{
    future::Future,
    ops::{Index, IndexMut},
    pin::Pin,
};

pub use filesystem::config::WearPos;
use filesystem::config::{EquipBonuses, WeaponCategory, WearFlag};
use macros::player_system;
use net::{InvEntry, InvType, if_events, if_set_events};
use persistence::player::PlayerData;

use crate::{
    player::{
        Clientbound, Obj, PlayerSnapshot,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
        ui::tabs,
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
    player: PlayerHandle,
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

    pub fn bonuses(&self) -> EquipBonuses {
        let mut total = EquipBonuses::default();
        for obj in self.slots.0.iter().flatten() {
            let Some(t) = crate::provider::get_obj_type(obj.id as u32) else { continue };
            let b = &t.equip;
            total.atk_stab = total.atk_stab.saturating_add(b.atk_stab);
            total.atk_slash = total.atk_slash.saturating_add(b.atk_slash);
            total.atk_crush = total.atk_crush.saturating_add(b.atk_crush);
            total.atk_magic = total.atk_magic.saturating_add(b.atk_magic);
            total.atk_ranged = total.atk_ranged.saturating_add(b.atk_ranged);
            total.def_stab = total.def_stab.saturating_add(b.def_stab);
            total.def_slash = total.def_slash.saturating_add(b.def_slash);
            total.def_crush = total.def_crush.saturating_add(b.def_crush);
            total.def_magic = total.def_magic.saturating_add(b.def_magic);
            total.def_ranged = total.def_ranged.saturating_add(b.def_ranged);
            total.str_bonus = total.str_bonus.saturating_add(b.str_bonus);
            total.ranged_str = total.ranged_str.saturating_add(b.ranged_str);
            total.magic_dmg = total.magic_dmg.saturating_add(b.magic_dmg);
            total.prayer = total.prayer.saturating_add(b.prayer);
        }
        total
    }

    pub fn weapon_category(&self) -> Option<WeaponCategory> {
        self.slots[WearPos::Weapon]
            .and_then(|obj| crate::provider::get_obj_type(obj.id as u32))
            .and_then(|t| t.weapon_category)
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

    async fn send_if_set_events(&mut self) {
        for component in [8u16, 11, 14, 17, 20, 23, 26, 29, 32, 35, 38] {
            self.player
                .if_set_events(if_set_events!(
                    interface_id: tabs::EQUIPMENT,
                    component_id: component,
                    slots: [0 => 0],
                    right_click[0]
                ))
                .await;
        }

        self.player
            .if_set_events(if_set_events!(
                interface_id: tabs::EQUIPMENT,
                component_id: 4,
                slots: [0 => 0],
                right_click[0]
            ))
            .await;
    }

    pub async fn flush(&mut self) {
        let objs = self
            .slots
            .0
            .iter()
            .map(|s| {
                s.map(|obj| InvEntry {
                    obj_id: obj.id,
                    amount: obj.amount,
                })
            })
            .collect();
        self.player.update_inv(InvType::Worn, false, objs).await;
    }
}

#[player_system]
impl PlayerSystem for Worn {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            slots: WornSlots::from(&ctx.player_data.worn),
        }
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.flush().await;
            self.send_if_set_events().await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.worn = self.slots.0.iter().map(|s| s.map(|obj| (obj.id, obj.amount))).collect();
    }
}
