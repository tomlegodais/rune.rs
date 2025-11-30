use crate::account::Account;
use crate::message::{GameScene, RootInterface};
use crate::player::{Connection, Scene};
use crate::world::{Position, RegionId};
use tracing::info;

pub struct Player {
    pub id: u16,
    pub _account_id: i64,
    pub username: String,
    pub _rights: u8,

    pub connection: Connection,
    pub position: Position,
    pub current_region: RegionId,
    pub scene: Scene,
}

impl Player {
    pub fn new(id: u16, account: &Account, connection: Connection, position: Position) -> Self {
        let scene = Scene::new(position, 0);
        let current_region = position.region_id();

        Self {
            id,
            _account_id: account.id,
            username: account.username.clone(),
            _rights: account.rights,
            connection,
            position,
            current_region,
            scene,
        }
    }

    pub async fn on_login(&mut self) {
        self.send_game_scene(true).await;
        self.send_root_interface(746).await;

        info!("Player ({}) logged in", self.username);
    }

    pub async fn tick(&mut self) {
        if self.scene.needs_rebuild(self.position) {
            self.scene.rebuild(self.position);
            self.send_game_scene(false).await;
        }
    }

    async fn send_game_scene(&mut self, init: bool) {
        self.connection
            .send(GameScene {
                init,
                position: self.position,
                scene: &self.scene,
                player_id: self.id,
            })
            .await;
    }

    async fn send_root_interface(&mut self, root_id: u16) {
        self.connection.send(RootInterface { root_id }).await;
    }
}
