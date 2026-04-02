use std::{future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use persistence::player::PlayerData;

use crate::{
    player::{
        PlayerSnapshot,
        mask::AppearanceMask,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
        worn::WornSlots,
    },
    world::World,
};

pub const DEFAULT_READYANIM: u16 = 1426;

#[derive(Clone)]
pub struct Appearance {
    player: PlayerHandle,

    pub male: bool,
    pub look: [u16; 7],
    pub colors: [u8; 5],
}

impl Appearance {
    pub fn to_mask(&self, worn: &WornSlots) -> AppearanceMask {
        AppearanceMask {
            male: self.male,
            look: self.look,
            colors: self.colors,
            display_name: self.player.username.clone(),
            combat_level: self.player.stat().combat_level(),
            worn: *worn,
        }
    }

    pub fn flush(&mut self) {
        let mask = self.to_mask(self.player.worn().slots());
        self.player.player_info.add_mask(mask);
    }
}

#[player_system]
impl PlayerSystem for Appearance {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            male: ctx.player_data.male,
            look: ctx.player_data.look,
            colors: ctx.player_data.colors,
        }
    }

    fn dependencies() -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<crate::player::worn::Worn>()]
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.flush();
        })
    }

    fn tick_context(_: &Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.male = self.male;
        data.look = self.look;
        data.colors = self.colors;
    }
}
