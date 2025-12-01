use crate::account::Account;
use crate::message::GameScene;
use crate::player::{Connection, Scene, SharedConnection, WidgetManager};
use crate::world::{Position, RegionId};
use net::ServerMessage;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct Player {
    pub id: u16,
    pub _account_id: i64,
    pub username: String,
    pub _rights: u8,

    pub connection: SharedConnection,
    pub position: Position,
    pub current_region: RegionId,
    pub scene: Scene,
    pub widgets: WidgetManager,
}

impl Player {
    pub fn new(
        id: u16,
        account: &Account,
        connection: Connection,
        position: Position,
        display_mode: u8,
    ) -> Self {
        let shared_connection = Arc::new(Mutex::new(connection));
        let scene = Scene::new(position, 0);
        let current_region = position.region_id();
        let widgets = WidgetManager::new(Arc::clone(&shared_connection), display_mode);

        Self {
            id,
            _account_id: account.id,
            username: account.username.clone(),
            _rights: account.rights,
            connection: shared_connection,
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
        self.send_message(GameScene {
            init,
            position: self.position,
            scene: &self.scene,
            player_id: self.id,
        })
        .await;
    }

    pub async fn send_message(&self, msg: impl ServerMessage) {
        let mut connection = self.connection.lock().await;
        connection.send(msg).await;
    }
}
