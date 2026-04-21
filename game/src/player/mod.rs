#[macro_use]
mod macros;

mod action;
mod appearance;
mod clientbound;
mod combat;
mod dialogue;
mod gpi;
mod hitpoints;
mod info;
mod interaction;
mod interface;
mod inv;
mod loc;
mod mask;
mod movement;
mod obj;
mod objstack;
mod options;
mod stat;
mod state;
mod system;
mod ui;
mod varp;
mod viewport;
mod worn;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub use action::{
    ActionShared, ActionState, PlayerRef, SeqResetGuard, active_player, active_shared, await_dialogue,
    clear_action_context, delay, is_action_locked, lock, poll_action, send_message, set_action_context, unlock,
};
pub use appearance::{Appearance, DEFAULT_READYANIM};
pub use clientbound::Clientbound;
pub use dialogue::{DialogueEntity, OPTIONS_BASE, OPTIONS_FIRST_COMPONENT};
pub use gpi::encode_player_info;
pub use info::PlayerInfo;
pub use interaction::{InteractionTarget, resolve as resolve_interaction};
pub use interface::InterfaceSlot;
pub use inv::{SIZE as INV_SIZE, STACK_MAX};
pub use mask::{
    ChatMask, FaceDirectionMask, FaceEntityMask, Hit1Mask, Hit2Mask, MoveTypeMask, SeqMask, SpotAnim1Mask,
    SpotAnim2Mask, TempMoveTypeMask,
};
use net::{Inbox, Outbox};
pub use obj::Obj;
use persistence::{Account, PlayerData, Rights};
pub use stat::{NUM_STATS, Stat};
use system::{PlayerHandle, PlayerInitContext, SystemStore};
pub use ui::{chatbox, equipment};
pub use varp::VarpManager;
pub use viewport::Viewport;
pub use worn::{SIZE as WORN_SIZE, WornSlots};

use crate::{
    entity::{Entity, MaskBlock, MoveStep, Seq, SeqBuilder, SpotAnim, SpotAnimBuilder},
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
    pub worn: WornSlots,
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
    pub player_info: Box<PlayerInfo>,
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
        snapshots: &[PlayerSnapshot],
    ) -> Self {
        let username = account.display_name();
        let position = Position::new(player_data.x, player_data.y, player_data.plane);
        let viewport = Viewport::new(position, 0);
        let npc_info = NpcInfo::new(outbox.clone());
        let player_info = PlayerInfo::new(outbox.clone(), index, snapshots, &[&MoveTypeMask(player_data.running)]);

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
            systems: SystemStore::empty(),
        }
    }

    pub fn init_systems(&mut self, player_data: &PlayerData, display_mode: u8) {
        let handle = PlayerHandle::new(self as *mut _);
        self.systems.init(&PlayerInitContext {
            index: self.index,
            player_data: player_data.clone(),
            display_mode,
            display_name: self.username.clone(),
            player: handle,
        });
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
            worn: *self.worn().slots(),
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
            inv: vec![None; INV_SIZE],
            worn: vec![None; WORN_SIZE],
            combat_style: 0,
            auto_retaliate: true,
            spec_energy: 1000,
            current_hp: 10,
        };

        self.systems.for_each_persist(&mut data);
        data
    }

    pub async fn on_login(&mut self) {
        self.rebuild_normal(true).await;

        let systems = &mut self.systems as *mut SystemStore;
        unsafe { &mut *systems }.on_login(self).await;

        self.send_message("Welcome to RuneScape.").await;

        tracing::info!(index = self.index, username = self.username, "Player Logged In");
    }

    pub async fn tick_movement(&mut self, world: &Arc<World>) {
        let snapshot = self.snapshot();
        let systems = &mut self.systems as *mut SystemStore;
        unsafe { &mut *systems }
            .tick_phase(system::TickPhase::Movement, world, &snapshot)
            .await;
    }

    pub async fn tick_systems(&mut self, world: &Arc<World>) {
        let snapshot = self.snapshot();
        let systems = &mut self.systems as *mut SystemStore;
        unsafe { &mut *systems }
            .tick_phase(system::TickPhase::Default, world, &snapshot)
            .await;
    }

    pub async fn sync(
        &mut self,
        player_snapshots: &[PlayerSnapshot],
        npc_snapshots: &[NpcSnapshot],
        world: &Arc<World>,
    ) {
        if self.viewport.try_rebuild(self.position) {
            self.rebuild_normal(false).await;
            self.obj_stack_mut().on_viewport_rebuild(&world.obj_stacks).await;
            self.loc_mut().on_viewport_rebuild(&world.locs).await;
        }

        let player_pos = self.position;
        let viewport = &self.viewport;

        self.player_info
            .sync(player_snapshots, |pos| viewport.is_within_view(player_pos, pos));

        self.npc_info
            .sync(npc_snapshots, |pos| viewport.is_within_view(player_pos, pos));
    }

    pub async fn cancel_action(&mut self, close_interfaces: bool) {
        self.world().action_states.lock().remove(&self.index);
        self.combat_mut().set_combat_target(None);
        self.dialogue_mut().close().await;
        if close_interfaces {
            self.interface_mut().close_slot(InterfaceSlot::Modal).await;
            self.interface_mut().close_slot(InterfaceSlot::Inventory).await;
        }
    }

    pub fn has_seq(&self) -> bool {
        self.player_info.self_state().masks.has(mask::PlayerMask::SEQ)
    }

    pub fn hit(&mut self, hit: crate::entity::Hit) {
        if self.player_info.self_state().masks.has(mask::PlayerMask::HIT_1) {
            self.player_info.add_mask(Hit2Mask(hit));
        } else {
            self.player_info.add_mask(Hit1Mask(hit));
        }
    }

    pub fn seq(&mut self, id: u16) -> SeqBuilder<impl FnOnce(Seq) + '_> {
        SeqBuilder::new(id, |a| self.player_info.add_mask(SeqMask(a)))
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
