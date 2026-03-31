use std::{future::Future, pin::Pin};

use macros::player_system;
use net::{Outbox, OutboxExt, SetPlayerOp};

use crate::{
    player::{
        PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
    },
    world::World,
};

const NUM_OPTIONS: usize = 5;

pub struct SetPlayerOps {
    outbox: Outbox,
    options: [Option<(String, bool)>; NUM_OPTIONS],
}

impl SetPlayerOps {
    fn new(outbox: Outbox) -> Self {
        let mut options: [Option<(String, bool)>; NUM_OPTIONS] = Default::default();
        options[1] = Some(("Follow".into(), false));
        options[2] = Some(("Trade with".into(), false));
        options[4] = Some(("Report".into(), false));
        Self { outbox, options }
    }

    pub async fn set(&mut self, slot: u8, option: impl Into<String>, top: bool) {
        let idx = slot as usize;
        self.options[idx] = Some((option.into(), top));
        self.send_slot(slot).await;
    }

    pub async fn clear(&mut self, slot: u8) {
        let idx = slot as usize;
        self.options[idx] = None;
        self.outbox
            .write(SetPlayerOp {
                slot,
                top: false,
                op: "null".into(),
            })
            .await;
    }

    async fn flush(&mut self) {
        for slot in 0..NUM_OPTIONS as u8 {
            self.send_slot(slot).await;
        }
    }

    async fn send_slot(&mut self, slot: u8) {
        let Some((ref text, top)) = self.options[slot as usize] else {
            return;
        };
        let text = text.clone();
        self.outbox.write(SetPlayerOp { slot, top, op: text }).await;
    }
}

#[player_system]
impl PlayerSystem for SetPlayerOps {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self::new(ctx.outbox.clone())
    }

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.flush())
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}
}
