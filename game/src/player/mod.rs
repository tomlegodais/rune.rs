#[macro_use]
mod macros;

mod action;
mod appearance;
mod equipment;
mod gpi;
mod grounditem;
mod info;
mod interaction;
mod interface;
mod inventory;
mod item;
mod mask;
mod movement;
mod options;
mod skill;
mod state;
mod system;
mod ui;
mod varp;
mod viewport;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub use action::{
    ActionShared, ActionState, AnimResetGuard, PlayerRef, active_player, active_shared, clear_action_context, delay,
    is_action_locked, lock, poll_action, send_message, set_action_context, unlock,
};
pub use appearance::{Appearance, DEFAULT_RENDER_EMOTE};
pub use equipment::{EquipSlots, SIZE as EQUIPMENT_SIZE};
pub use gpi::encode_player_info;
pub use info::PlayerInfo;
pub use interaction::{InteractionTarget, resolve as resolve_interaction};
pub use interface::SubInterface;
pub use inventory::SIZE as INVENTORY_SIZE;
pub use item::Item;
pub use mask::{
    AnimationMask, ChatMask, FaceDirectionMask, MoveTypeMask, SpotAnim1Mask, SpotAnim2Mask, TempMoveTypeMask,
};
pub use movement::{Movement, MovementContext};
use net::{ChatMessage, Inbox, Logout, Outbox, OutboxExt};
use persistence::{
    account::{Account, Rights},
    player::PlayerData,
};
pub use skill::Skill;
use system::{PlayerInitContext, SystemStore};
use tracing::info;
pub use varp::VarpManager;
pub use viewport::Viewport;

use crate::{
    entity::{Anim, AnimBuilder, Entity, MaskBlock, MoveStep, SpotAnim, SpotAnimBuilder},
    npc::{NpcInfo, NpcSnapshot},
    world::{Direction, Position, Teleport, World},
};

#[derive(Clone)]
pub struct PlayerSnapshot {
    pub index: usize,
    pub position: Position,
    pub region_base: Position,
    pub face_direction: Direction,
    pub appearance: Appearance,
    pub equipment: EquipSlots,
    pub masks: MaskBlock,
    pub teleport: Option<Teleport>,
    pub move_step: MoveStep,
    pub running: bool,
}

pub struct Player {
    pub entity: Entity,
    pub player_id: i64,
    pub account_id: i64,
    pub username: String,
    pub rights: Rights,

    pub inbox: Inbox,
    pub outbox: Outbox,

    pub viewport: Viewport,
    pub player_info: PlayerInfo,
    pub npc_info: NpcInfo,
    pub systems: SystemStore,
}

impl Player {
    pub fn new(
        index: usize,
        account: &Account,
        player_data: &PlayerData,
        inbox: Inbox,
        outbox: Outbox,
        display_mode: u8,
        snapshots: &[PlayerSnapshot],
    ) -> Self {
        let username = account.display_name();
        let position = Position::new(player_data.x, player_data.y, player_data.plane);
        let viewport = Viewport::new(outbox.clone(), position, 0);
        let npc_info = NpcInfo::new(outbox.clone());
        let player_info = PlayerInfo::new(outbox.clone(), index, snapshots, &[&MoveTypeMask(player_data.running)]);

        let systems = SystemStore::from_init(&PlayerInitContext {
            index,
            outbox: outbox.clone(),
            player_data: player_data.clone(),
            display_mode,
            display_name: username.clone(),
        });

        Self {
            entity: Entity::new(index, position),
            player_id: player_data.player_id,
            account_id: account.id,
            username,
            rights: account.rights,
            inbox,
            outbox,
            viewport,
            player_info,
            npc_info,
            systems,
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
            appearance: self.appearance().clone(),
            equipment: *self.equipment().slots(),
            masks: state.masks.clone(),
            teleport: state.teleport,
            move_step: state.move_step,
            running,
        }
    }

    pub fn to_player_data(&self) -> PlayerData {
        let mut data = PlayerData {
            player_id: self.player_id,
            x: self.position.x,
            y: self.position.y,
            plane: self.position.plane,
            running: false,
            run_energy: 0,
            male: true,
            look: [0; 7],
            colors: [0; 5],
            levels: [1; 24],
            xp: [0; 24],
            inventory: vec![None; INVENTORY_SIZE],
            equipment: vec![None; EQUIPMENT_SIZE],
        };

        self.systems.for_each_persist(&mut data);
        data
    }

    pub async fn on_login(&mut self) {
        self.viewport
            .send_game_scene(true, self.index, &self.player_info, self.position)
            .await;

        self.systems.on_login(&mut self.player_info).await;
        self.send_message("Welcome to RuneScape.").await;

        info!("Player (index={}, username={}) logged in", self.index, self.username);
    }

    pub async fn tick_systems(&mut self, world: &Arc<World>) {
        self.systems.tick(world, &self.snapshot()).await;
    }

    pub async fn sync(
        &mut self,
        player_snapshots: &[PlayerSnapshot],
        npc_snapshots: &[NpcSnapshot],
        world: &Arc<World>,
    ) {
        if self
            .viewport
            .try_rebuild(self.position, self.index, &self.player_info)
            .await
        {
            self.ground_item_mut().on_viewport_rebuild(&world.ground_items).await;
        }

        let player_pos = self.position;
        let viewport = &self.viewport;

        self.player_info
            .sync(player_snapshots, |pos| viewport.is_within_view(player_pos, pos));

        self.npc_info
            .sync(npc_snapshots, |pos| viewport.is_within_view(player_pos, pos));
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

    pub fn flush_appearance(&mut self) {
        let mask = self.appearance().to_mask(self.equipment().slots());
        self.player_info.add_mask(mask);
    }

    pub fn anim(&mut self, id: u16) -> AnimBuilder<impl FnOnce(Anim) + '_> {
        AnimBuilder::new(id, |a| self.player_info.add_mask(AnimationMask(a)))
    }

    pub fn spot_anim(&mut self, id: u16) -> SpotAnimBuilder<impl FnOnce(SpotAnim) + '_> {
        SpotAnimBuilder::new(id, |sa| {
            if self.player_info.self_state().masks.has(mask::PlayerMask::SPOT_ANIM_1) {
                self.player_info.add_mask(SpotAnim2Mask(sa));
            } else {
                self.player_info.add_mask(SpotAnim1Mask(sa));
            }
        })
    }

    pub async fn toggle_run(&mut self) {
        let running = !self.movement().running;
        with_movement!(self, |m, ctx| m.set_run(&mut ctx, running).await);
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
