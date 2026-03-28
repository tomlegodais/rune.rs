#[macro_use]
mod macros;

mod action;
mod appearance;
mod gpi;
mod grounditem;
mod info;
mod interaction;
mod interface;
mod inventory;
mod mask;
mod movement;
mod options;
mod skill;
mod state;
mod system;
mod ui;
mod varp;
mod viewport;

pub(crate) use action::{
    ActionShared, ActionState, SkillActionBuilder, active_player, active_shared,
    clear_action_context, delay, is_action_locked, npc_force_talk, poll_action, send_message,
    set_action_context,
};
pub(crate) use appearance::Appearance;
pub(crate) use info::PlayerInfo;
pub(crate) use interaction::{InteractionTarget, resolve as resolve_interaction};
pub(crate) use interface::SubInterface;
pub(crate) use inventory::SIZE as INVENTORY_SIZE;
pub(crate) use mask::{
    AnimationMask, AppearanceMask, ChatMask, FaceDirectionMask, MoveTypeMask, SpotAnim1Mask,
    SpotAnim2Mask, TempMoveTypeMask,
};
pub(crate) use movement::{Movement, MovementContext};
pub(crate) use skill::Skill;
pub(crate) use varp::VarpManager;
pub(crate) use viewport::Viewport;

use crate::entity::{Anim, AnimBuilder, Entity, MaskBlock, MoveStep, SpotAnim, SpotAnimBuilder};
use crate::npc::{NpcInfo, NpcSnapshot};
use crate::world::{Direction, Position, Teleport};
use net::{ChatMessage, Inbox, Logout, Outbox, OutboxExt};
use persistence::account::{Account, Rights};
use persistence::player::PlayerData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use system::{PlayerInitContext, SystemStore};
use tracing::info;

#[derive(Clone)]
pub struct PlayerSnapshot {
    pub index: usize,
    pub position: Position,
    pub region_base: Position,
    pub face_direction: Direction,
    pub appearance: Appearance,
    pub masks: MaskBlock,
    pub teleport: Option<Teleport>,
    pub move_step: MoveStep,
    pub running: bool,
}

pub struct Player {
    pub entity: Entity,
    pub player_data_id: i64,
    pub account_id: i64,
    pub username: String,
    pub rights: Rights,

    pub inbox: Inbox,
    pub outbox: Outbox,

    pub appearance: Appearance,
    pub viewport: Viewport,
    pub player_info: PlayerInfo,
    pub npc_info: NpcInfo,
    pub systems: SystemStore,
}

impl Player {
    pub fn new(
        index: usize,
        account: &Account,
        data: &PlayerData,
        inbox: Inbox,
        outbox: Outbox,
        display_mode: u8,
        snapshots: &[PlayerSnapshot],
    ) -> Self {
        let username = account.display_name();
        let position = Position::new(data.x, data.y, data.plane);
        let viewport = Viewport::new(outbox.clone(), position, 0);
        let appearance = Appearance::from_data(&username, data.male, data.look, data.colors);
        let npc_info = NpcInfo::new(outbox.clone());
        let player_info = PlayerInfo::new(
            outbox.clone(),
            index,
            snapshots,
            &[
                &MoveTypeMask(data.running),
                &AppearanceMask::new(&appearance),
            ],
        );

        let init_ctx = PlayerInitContext {
            index,
            outbox: outbox.clone(),
            data: data.clone(),
            display_mode,
        };

        Self {
            entity: Entity::new(index, position),
            player_data_id: data.player_id,
            account_id: account.id,
            username,
            rights: account.rights,
            inbox,
            outbox,
            viewport,
            player_info,
            npc_info,
            appearance,
            systems: SystemStore::from_init(&init_ctx),
        }
    }

    pub fn snapshot(&self) -> PlayerSnapshot {
        let state = self.player_info.self_state();
        let running = self.movement().running;

        PlayerSnapshot {
            index: self.index,
            position: self.position,
            region_base: self.viewport.region_base,
            face_direction: self.face_direction,
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
            inventory: vec![None; 28],
        };

        self.systems.for_each_persist(&mut data);
        data
    }

    pub async fn on_login(&mut self) {
        self.viewport
            .send_game_scene(true, self.index, &self.player_info, self.position)
            .await;

        self.systems.on_login().await;
        self.send_message("Welcome to RuneScape.").await;

        info!(
            "Player (index={}, username={}) logged in",
            self.index, self.username
        );
    }

    pub async fn tick_systems(&mut self, world: &Arc<crate::world::World>) {
        self.systems.tick(world, &self.snapshot()).await;
    }

    pub async fn sync(
        &mut self,
        player_snapshots: &[PlayerSnapshot],
        npc_snapshots: &[NpcSnapshot],
        world: &Arc<crate::world::World>,
    ) {
        if self
            .viewport
            .try_rebuild(self.position, self.index, &self.player_info)
            .await
        {
            self.ground_item_mut()
                .on_viewport_rebuild(&world.ground_items)
                .await;
        }

        let player_pos = self.position;
        let viewport = &self.viewport;

        self.player_info.sync(player_snapshots, |pos| {
            viewport.is_within_view(player_pos, pos)
        });

        self.npc_info.sync(npc_snapshots, |pos| {
            viewport.is_within_view(player_pos, pos)
        });
    }

    pub async fn send_message(&mut self, text: &str) {
        self.outbox
            .write(ChatMessage {
                msg_type: 0,
                text: text.to_string(),
            })
            .await;
    }

    pub async fn logout(&mut self) {
        self.outbox.write(Logout).await;
    }

    pub fn anim(&mut self, id: u16) -> AnimBuilder<impl FnOnce(Anim) + '_> {
        AnimBuilder::new(id, |a| self.player_info.add_mask(AnimationMask(a)))
    }

    pub fn spot_anim(&mut self, id: u16) -> SpotAnimBuilder<impl FnOnce(SpotAnim) + '_> {
        SpotAnimBuilder::new(id, |sa| {
            if self
                .player_info
                .self_state()
                .masks
                .has(mask::PlayerMask::SPOT_ANIM_1)
            {
                self.player_info.add_mask(SpotAnim2Mask(sa));
            } else {
                self.player_info.add_mask(SpotAnim1Mask(sa));
            }
        })
    }

    pub fn reset(&mut self) {
        self.player_info.reset();
        self.npc_info.reset();
    }
}

impl Deref for Player {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.entity
    }
}

impl DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}
