pub const DEFAULT_READYANIM: u16 = 1426;

use std::{future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use persistence::player::PlayerData;

use crate::{
    player::{
        PlayerSnapshot,
        mask::AppearanceMask,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
        worn::{Worn, WornSlots},
    },
    world::World,
};

#[derive(Clone)]
pub struct Appearance {
    pub male: bool,
    pub look: [u16; 7],
    pub colors: [u8; 5],
    pub display_name: String,
    pub combat_level: u8,
}

impl Appearance {
    pub fn to_mask(&self, worn: &WornSlots) -> AppearanceMask {
        AppearanceMask {
            male: self.male,
            look: self.look,
            colors: self.colors,
            display_name: self.display_name.clone(),
            combat_level: self.combat_level,
            worn: *worn,
        }
    }
}

#[player_system]
impl PlayerSystem for Appearance {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            male: ctx.player_data.male,
            look: ctx.player_data.look,
            colors: ctx.player_data.colors,
            display_name: ctx.display_name.clone(),
            combat_level: 3,
        }
    }

    fn dependencies() -> Vec<std::any::TypeId> {
        vec![std::any::TypeId::of::<Worn>()]
    }

    fn on_login<'a>(&'a mut self, ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let worn = ctx.take::<Worn>();
            ctx.player_info.add_mask(self.to_mask(worn.slots()));
            ctx.put_back(worn);
        })
    }

    fn tick_context(_: &Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.male = self.male;
        data.look = self.look;
        data.colors = self.colors;
    }
}
