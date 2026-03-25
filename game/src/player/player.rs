use crate::player::movement::Movement;
use crate::player::state::MoveStep;
use crate::player::system::{PlayerInitContext, PlayerSystem, SystemContextFields, SystemStore};
use crate::player::{
    Appearance, AppearanceMask, MaskBlock, MoveTypeMask, PlayerInfo, Viewport, gpi,
};
use crate::world::{Position, RegionId, Teleport};
use net::{ChatMessage, GameScene, Inbox, Logout, Outbox, OutboxExt};
use persistence::account::{Account, Rights};
use persistence::player::PlayerData;
use std::array;
use tracing::info;

#[derive(Clone)]
pub struct PlayerSnapshot {
    pub id: usize,
    pub position: Position,
    pub appearance: Appearance,
    pub masks: MaskBlock,
    pub teleport: Option<Teleport>,
    pub move_step: MoveStep,
    pub running: bool,
}

pub struct Player {
    pub id: usize,
    pub player_data_id: i64,
    pub _account_id: i64,
    pub username: String,
    pub rights: Rights,

    pub inbox: Inbox,
    pub outbox: Outbox,
    pub position: Position,
    pub current_region: RegionId,
    pub viewport: Viewport,
    pub player_info: PlayerInfo,
    pub appearance: Appearance,

    pub systems: SystemStore,
}

impl Player {
    pub fn new(
        id: usize,
        account: &Account,
        data: &PlayerData,
        inbox: Inbox,
        outbox: Outbox,
        display_mode: u8,
        snapshots: &[PlayerSnapshot],
    ) -> Self {
        let username = account.display_name();
        let position = Position::new(data.x, data.y, data.plane);
        let viewport = Viewport::new(position, 0);
        let appearance = Appearance::from_data(&username, data.male, data.look, data.colors);
        let player_info = PlayerInfo::new(
            id,
            snapshots,
            &[
                &MoveTypeMask(data.running),
                &AppearanceMask::new(&appearance),
            ],
        );

        let init_ctx = PlayerInitContext {
            outbox: outbox.clone(),
            data: data.clone(),
            display_mode,
        };

        Self {
            id,
            player_data_id: data.player_id,
            _account_id: account.id,
            username,
            rights: account.rights,
            inbox,
            outbox,
            position,
            current_region: position.region_id(),
            viewport,
            player_info,
            appearance,
            systems: SystemStore::from_init(&init_ctx),
        }
    }

    pub fn system<T: PlayerSystem>(&self) -> &T {
        self.systems.get::<T>()
    }

    #[allow(dead_code)]
    pub fn system_mut<T: PlayerSystem>(&mut self) -> &mut T {
        self.systems.get_mut::<T>()
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        let state = self.player_info.self_state();
        let running = self.systems.get::<Movement>().running;

        PlayerSnapshot {
            id: self.id,
            position: self.position,
            appearance: self.appearance.clone(),
            masks: state.masks.clone(),
            teleport: state.teleport,
            move_step: state.move_step,
            running,
        }
    }

    pub fn to_player_data(&self) -> PlayerData {
        let mut data = PlayerData {
            player_id: self.player_data_id,
            x: self.position.x,
            y: self.position.y,
            plane: self.position.plane,
            running: false,
            run_energy: 0,
            male: self.appearance.male,
            look: self.appearance.look,
            colors: self.appearance.colors,
            levels: [1; 24],
            xp: [0; 24],
        };
        self.systems.for_each_persist(&mut data);
        data
    }

    pub async fn on_login(&mut self) {
        self.send_game_scene(true).await;

        self.systems
            .on_login(&mut SystemContextFields {
                outbox: &self.outbox,
                position: &mut self.position,
                player_info: &mut self.player_info,
                viewport: &self.viewport,
                appearance: &self.appearance,
            })
            .await;

        self.send_message("Welcome to RuneScape.").await;

        info!(
            "Player (id={}, username={}) logged in",
            self.id, self.username
        );
    }

    pub async fn tick(&mut self, snapshots: &[PlayerSnapshot]) {
        if self.viewport.needs_rebuild(self.position) {
            self.viewport.rebuild(self.position);
            self.send_game_scene(false).await;
        }

        let viewport = &self.viewport;
        self.player_info
            .sync(snapshots, |pos| viewport.is_within_view(pos));
    }

    pub async fn logout(&mut self) {
        self.outbox.write(Logout).await;
    }

    pub fn reset(&mut self) {
        self.player_info.reset();
    }

    pub async fn send_player_info(&mut self) {
        let frame = gpi::encode(&mut self.player_info);
        let _ = self.outbox.send(frame).await;
    }

    pub async fn send_message(&mut self, text: &str) {
        self.outbox
            .write(ChatMessage {
                msg_type: 0,
                text: text.to_string(),
            })
            .await;
    }

    async fn send_game_scene(&mut self, init: bool) {
        self.outbox
            .write(GameScene {
                init,
                position_bits: self.position.to_bits(),
                player_id: self.id,
                view_distance: self.viewport.view_distance,
                chunk_x: self.position.chunk_x(),
                chunk_y: self.position.chunk_y(),
                region_count: self.viewport.region_ids().len(),
                region_hashes: array::from_fn(|i| self.player_info[i].region_hash),
            })
            .await;
    }
}
