use crate::player::state::MoveStep;
use crate::player::{
    gpi, Appearance, AppearanceMask, MaskBlock, MoveTypeMask, PlayerInfo,
    SkillManager, TempMoveTypeMask, Viewport, WidgetManager,
};
use crate::world::{running_direction, Position, RegionId, Teleport};
use net::{ChatMessage, GameScene, Inbox, Outbox, OutboxExt};
use persistence::account::{Account, Rights};
use persistence::player::PlayerData;
use std::array;
use std::collections::VecDeque;
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
    pub skills: SkillManager,
    pub widgets: WidgetManager,
    pub appearance: Appearance,
    pub walk_queue: VecDeque<Position>,
    pub running: bool,
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
        let current_region = position.region_id();
        let skills = SkillManager::from_data(outbox.clone(), data.levels, data.xp);
        let widgets = WidgetManager::new(outbox.clone(), display_mode);
        let appearance = Appearance::from_data(&username, data.male, data.look, data.colors);
        let player_info = PlayerInfo::new(
            id,
            snapshots,
            &[
                &MoveTypeMask(data.running),
                &AppearanceMask::new(&appearance),
            ],
        );

        Self {
            id,
            player_data_id: data.player_id,
            _account_id: account.id,
            username,
            rights: account.rights,
            inbox,
            outbox,
            position,
            current_region,
            viewport,
            player_info,
            skills,
            widgets,
            appearance,
            walk_queue: VecDeque::new(),
            running: data.running,
        }
    }

    pub fn to_player_data(&self) -> PlayerData {
        PlayerData {
            player_id: self.player_data_id,
            x: self.position.x,
            y: self.position.y,
            plane: self.position.plane,
            running: self.running,
            male: self.appearance.male,
            look: self.appearance.look,
            colors: self.appearance.colors,
            levels: self.skills.levels(),
            xp: self.skills.xp_values(),
        }
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        let state = self.player_info.self_state();

        PlayerSnapshot {
            id: self.id,
            position: self.position,
            appearance: self.appearance.clone(),
            masks: state.masks.clone(),
            teleport: state.teleport,
            move_step: state.move_step,
            running: self.running,
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

    pub fn walk_to(&mut self, dest: Position, force_run: bool) {
        self.walk_queue.clear();
        self.running = force_run;

        let mut current = self.position;
        while current != dest {
            let Some(dir) = current.direction_to(dest) else {
                break;
            };
            current = current.step(dir);
            self.walk_queue.push_back(current);
        }
    }

    pub fn process_movement(&mut self) {
        let Some(next) = self.walk_queue.pop_front() else {
            return;
        };
        let Some(walk_dir) = self.position.direction_to(next) else {
            return;
        };

        self.position = next;

        if self.running {
            if let Some(run_pos) = self.walk_queue.pop_front() {
                if let Some(run_dir) = self.position.direction_to(run_pos) {
                    if let Some(opcode) = running_direction(walk_dir, run_dir) {
                        self.position = run_pos;
                        self.player_info.set_move_step(MoveStep::Run(opcode));
                        self.player_info.add_mask(TempMoveTypeMask::Run);
                        return;
                    }
                }
            }
        }

        self.player_info.set_move_step(MoveStep::Walk(walk_dir));
        self.player_info.add_mask(TempMoveTypeMask::Walk);
    }

    pub fn teleport(&mut self, destination: Position) {
        self.walk_queue.clear();
        self.player_info.teleport(Teleport {
            from: self.position,
            to: destination,
        });
        self.player_info.add_mask(TempMoveTypeMask::Teleport);
        self.position = destination;
    }

    pub async fn send_player_info(&mut self) {
        let frame = gpi::encode(&mut self.player_info);
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
                region_hashes: array::from_fn(|i| self.player_info[i].region_hash),
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
    }
}
