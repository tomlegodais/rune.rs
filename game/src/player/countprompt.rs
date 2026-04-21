use std::{future::Future, pin::Pin};

use macros::player_system;
use net::ScriptArg;

use crate::{
    player::{
        Clientbound, PlayerSnapshot,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::World,
};

pub type ResumeCountFn = for<'a> fn(&'a mut super::Player, u32) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub struct CountPrompt {
    player: PlayerHandle,
    pending: Option<ResumeCountFn>,
}

impl CountPrompt {
    pub async fn prompt(&mut self, text: impl Into<String> + Send, resume: ResumeCountFn) {
        self.pending = Some(resume);
        self.player
            .run_client_script(108, vec![ScriptArg::Str(text.into())])
            .await;
    }

    pub fn take(&mut self) -> Option<ResumeCountFn> {
        self.pending.take()
    }

    pub async fn clear(&mut self) {
        if self.pending.take().is_none() {
            return;
        }
        self.player.run_client_script(101, vec![]).await;
    }
}

#[player_system]
impl PlayerSystem for CountPrompt {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            pending: None,
        }
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}
}
