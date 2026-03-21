use crate::account::Account;
use crate::player::{
    Appearance, AppearanceEncoder, PlayerInfo, MaskBlock, MaskFlags, MoveTypeMask, SkillManager,
    Viewport, WidgetManager, gpi,
};
use crate::world::{Position, RegionId, Teleport};
use net::{ChatMessage, GameScene, Inbox, Outbox, OutboxExt};
use tracing::info;

#[derive(Clone)]
pub struct PlayerSnapshot {
    pub id: usize,
    pub position: Position,
    pub appearance: Appearance,
    pub mask_flags: MaskFlags,
    pub teleport: Option<Teleport>,
}

pub struct Player {
    pub id: usize,
    pub _account_id: i64,
    pub username: String,
    pub _rights: u8,

    pub inbox: Inbox,
    pub outbox: Outbox,
    pub position: Position,
    pub current_region: RegionId,
    pub viewport: Viewport,
    pub player_info: PlayerInfo,
    pub skills: SkillManager,
    pub widgets: WidgetManager,
    pub appearance: Appearance,
    pub mask_flags: MaskFlags,
}

impl Player {
    pub fn new(
        id: usize,
        account: &Account,
        inbox: Inbox,
        outbox: Outbox,
        position: Position,
        display_mode: u8,
        snapshots: &[PlayerSnapshot],
    ) -> Self {
        let viewport = Viewport::new(position, 0);
        let player_info = PlayerInfo::new(id, snapshots);
        let current_region = position.region_id();
        let skills = SkillManager::new(outbox.clone());
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
            viewport,
            player_info,
            skills,
            widgets,
            appearance: Appearance::new(&account.username, 3),
            mask_flags: MaskFlags::APPEARANCE | MaskFlags::MOVE_TYPE,
        }
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot {
            id: self.id,
            position: self.position,
            appearance: self.appearance.clone(),
            mask_flags: self.mask_flags,
            teleport: self.player_info.self_state().teleport,
        }
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

    pub async fn on_login(&mut self) {
        self.send_game_scene(true).await;
        self.widgets.on_login().await;
        self.skills.flush().await;
        self.send_message("Welcome to RuneScape.").await;

        info!(
            "Player (id={}, username={}) logged in",
            self.id, self.username
        );
    }

    pub fn teleport(&mut self, destination: Position) {
        self.player_info.self_state_mut().teleport = Some(Teleport {
            from: self.position,
            to: destination,
        });
        self.position = destination;
    }

    pub async fn send_player_info(&mut self) {
        let appearance_encoder = AppearanceEncoder::new(&self.appearance);
        let block = MaskBlock {
            flags: &self.mask_flags,
            move_type: &MoveTypeMask,
            appearance: &appearance_encoder,
        };
        let frame = gpi::encode(&mut self.player_info, &block);
        let _ = self.outbox.send(frame).await;
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
                region_hashes: core::array::from_fn(|i| self.player_info[i].region_hash),
            })
            .await;
    }

    pub async fn send_message(&mut self, text: &str) {
        self.outbox
            .write(ChatMessage {
                msg_type: 0,
                text: text.to_string(),
            })
            .await;
    }

    pub fn reset(&mut self) {
        self.player_info.reset();
        self.mask_flags.clear();
    }
}
