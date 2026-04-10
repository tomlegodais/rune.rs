use std::{future::Future, pin::Pin};

use macros::player_system;

use crate::{
    player::{
        Clientbound, PlayerSnapshot,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::World,
};

const NUM_OPTIONS: usize = 6;

pub struct SetPlayerOps {
    player: PlayerHandle,
    options: [Option<(String, bool)>; NUM_OPTIONS],
}

impl SetPlayerOps {
    pub async fn set(&mut self, slot: u8, option: impl Into<String>, top: bool) {
        let idx = slot as usize;
        self.options[idx] = Some((option.into(), top));
        self.send_player_op(slot).await;
    }

    pub async fn clear(&mut self, slot: u8) {
        let idx = slot as usize;
        self.options[idx] = None;
        self.player.set_player_op(slot, false, "null").await;
    }

    async fn flush(&mut self) {
        for slot in 0..NUM_OPTIONS as u8 {
            self.send_player_op(slot).await;
        }
    }

    async fn send_player_op(&mut self, slot: u8) {
        let Some((ref text, top)) = self.options[slot as usize] else {
            return;
        };
        self.player.set_player_op(slot, top, text.clone()).await;
    }
}

#[player_system]
impl PlayerSystem for SetPlayerOps {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        let mut options: [Option<(String, bool)>; NUM_OPTIONS] = Default::default();
        options[1] = Some(("Attack".into(), true));
        options[2] = Some(("Follow".into(), false));
        options[4] = Some(("Trade with".into(), false));
        options[5] = Some(("Req Assist".into(), false));
        Self {
            player: ctx.player,
            options,
        }
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.flush())
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}
}
