use crate::account::Account;
use crate::message::{GameScene, Inbox, Outbox, OutboxExt};
use crate::player::{Scene, WidgetManager};
use crate::world::{Position, RegionId};
use net::GameMessage;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct Player {
    pub id: u16,
    pub _account_id: i64,
    pub username: String,
    pub _rights: u8,

    pub inbox: Inbox,
    pub outbox: Outbox,
    pub position: Position,
    pub current_region: RegionId,
    pub scene: Scene,
    pub widgets: WidgetManager,
}

impl Player {
    pub fn new(
        id: u16,
        account: &Account,
        inbox: Inbox,
        outbox: Outbox,
        position: Position,
        display_mode: u8,
    ) -> Self {
        let scene = Scene::new(position, 0);
        let current_region = position.region_id();
        let widgets = WidgetManager::new(outbox.clone(), display_mode);

        Self {
            id,
            _account_id: account.id,
            username: account.username.clone(),
            _rights: account.rights,
            inbox,
            outbox,
            position,
            current_region,
            scene,
            widgets,
        }
    }

    pub async fn on_login(&mut self) {
        self.send_game_scene(true).await;
        self.widgets.on_login().await;

        info!("Player ({}) logged in", self.username);
    }

    pub async fn tick(&mut self) {
        if self.scene.needs_rebuild(self.position) {
            self.scene.rebuild(self.position);
            self.send_game_scene(false).await;
        }
    }

    async fn send_game_scene(&mut self, init: bool) {
        self.outbox
            .send_message(GameScene {
                init,
                position_bits: self.position.bits(),
                player_id: self.id,
                size: self.scene.size,
                center_chunk_x: self.scene.center_chunk_x,
                center_chunk_y: self.scene.center_chunk_y,
                region_count: self.scene.region_ids.len(),
            })
            .await;
    }

    pub fn drain(&mut self) -> Vec<GameMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.inbox.try_recv() {
            messages.push(msg);
        }
        messages
    }
}
