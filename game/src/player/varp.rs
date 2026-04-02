use std::{collections::HashMap, future::Future, pin::Pin};

use macros::player_system;

use crate::{
    player::{
        Clientbound,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    provider,
};

pub struct VarpManager {
    player: PlayerHandle,
    varps: HashMap<u16, i32>,
}

impl VarpManager {
    pub fn get(&self, id: u16) -> i32 {
        self.varps.get(&id).copied().unwrap_or(0)
    }

    pub async fn send_varp(&mut self, id: u16, value: i32) {
        self.varps.insert(id, value);
        if value >= i8::MIN as i32 && value <= i8::MAX as i32 {
            self.player.varp_small(id, value as u8).await;
        } else {
            self.player.varp_large(id, value as u32).await;
        }
    }

    pub async fn send_varbit(&mut self, id: u32, value: i32) {
        let Some(varbit) = provider::get_varbit_type(id) else {
            return;
        };

        let mask = varbit.mask() as i32;
        let current = self.get(varbit.varp);
        let updated = (current & !(mask << varbit.low_bit)) | ((value & mask) << varbit.low_bit);

        self.send_varp(varbit.varp, updated).await;

        if value <= u8::MAX as i32 {
            self.player.varbit_small(id as u16, value as u8).await;
        } else {
            self.player.varbit_large(id as u16, value as u32).await;
        }
    }
}

#[player_system]
impl PlayerSystem for VarpManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            varps: HashMap::new(),
        }
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            self.send_varp(281, 1000).await;
            self.send_varp(1160, -1).await;
            self.send_varp(1159, 1).await;
        })
    }

    fn tick_context(_: &std::sync::Arc<crate::world::World>, _: &crate::player::PlayerSnapshot) {}
}
