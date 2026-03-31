use std::sync::Arc;

use macros::player_system;
use net::ClickOption;

use crate::{
    handler::run_action,
    npc::FaceEntityMask as NpcFaceMask,
    player::{
        Player,
        mask::FaceEntityMask as PlayerFaceEntityMask,
        system::{PlayerInitContext, PlayerSystem},
    },
    world::{Position, World, can_interact_rect},
};

pub struct Interaction {
    pending: Option<PendingInteraction>,
}

pub struct PendingInteraction {
    pub target: InteractionTarget,
    pub option: ClickOption,
}

pub enum InteractionTarget {
    Loc {
        id: u16,
        x: i32,
        y: i32,
    },
    Npc {
        index: usize,
    },
    Player {
        index: usize,
    },
    Obj {
        slot: u16,
    },
    ObjStack {
        id: u32,
        position: Position,
    },
    Button {
        interface: u16,
        component: u16,
        option: ClickOption,
        slot1: u16,
        slot2: u16,
    },
}

impl Interaction {
    pub fn set(&mut self, target: InteractionTarget, option: ClickOption) {
        self.pending = Some(PendingInteraction { target, option });
    }

    pub fn clear(&mut self) {
        self.pending = None;
    }

    pub fn take(&mut self) -> Option<PendingInteraction> {
        self.pending.take()
    }

    pub fn pending(&self) -> Option<&PendingInteraction> {
        self.pending.as_ref()
    }
}

impl InteractionTarget {
    fn target_position(&self, world: &World, plane: i32) -> Option<Position> {
        match self {
            Self::Loc { x, y, .. } => Some(Position::new(*x, *y, plane)),
            Self::Npc { index } => world.npcs.contains(*index).then(|| world.npc(*index).position),
            Self::Player { index } => world.players.contains(*index).then(|| world.player(*index).position),
            Self::Obj { .. } => None,
            Self::ObjStack { position, .. } => Some(*position),
            Self::Button { .. } => None,
        }
    }
}

pub fn resolve(player: &mut Player, world: &World) {
    let player_index = player.index;

    let mut state = world.action_states.lock().remove(&player_index);
    if let Some(ref mut s) = state {
        if s.shared.delay_remaining.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            s.shared
                .delay_remaining
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }

        let shared = s.shared.clone();
        crate::player::action::set_action_context(player as *mut Player, shared);

        let poll_result = crate::player::action::poll_action(s);
        crate::player::action::clear_action_context();

        match poll_result {
            std::task::Poll::Ready(()) => {
                clear_face_if_needed(player, world);
            }
            std::task::Poll::Pending => {
                world.action_states.lock().insert(player_index, state.unwrap());
            }
        }
        return;
    }

    let Some(pending) = player.interaction().pending() else {
        clear_face_if_needed(player, world);
        return;
    };

    let Some(target_pos) = pending.target.target_position(world, player.position.plane) else {
        player.interaction_mut().clear();
        return;
    };

    let collision = crate::provider::get_collision();
    let is_adjacent = match &pending.target {
        InteractionTarget::Loc { id, .. } => {
            let (w, h, access) = collision.resolve_loc_params(target_pos, *id as u32);
            can_interact_rect(collision, player.position, target_pos, w, h, access)
        }
        InteractionTarget::Npc { index } => {
            let npc_id = world.npc(*index).npc_id;
            let size = crate::provider::get_npc_type(npc_id as u32)
                .map(|d| d.size as i32)
                .unwrap_or(1);
            can_interact_rect(collision, player.position, target_pos, size, size, 0)
        }
        InteractionTarget::Player { .. } => can_interact_rect(collision, player.position, target_pos, 1, 1, 0),
        InteractionTarget::Obj { .. } | InteractionTarget::Button { .. } => return,
        InteractionTarget::ObjStack { .. } => player.position == target_pos,
    };

    if !is_adjacent {
        if !player.entity.has_steps() {
            player.interaction_mut().clear();
        }
        return;
    }

    player.entity.stop();
    let pending = player.interaction_mut().take().unwrap();
    face_target(player, world, &pending.target, target_pos);

    if let Some(future) = crate::handler::dispatch(player, pending.target, pending.option) {
        run_action(player, future);
        if !world.action_states.lock().contains_key(&player_index) {
            clear_face_if_needed(player, world);
        }
    }
}

fn face_target(player: &mut Player, world: &World, target: &InteractionTarget, target_pos: Position) {
    if let Some(dir) = player.position.direction_to(target_pos) {
        player.entity.face_direction = dir;
    }

    match target {
        InteractionTarget::Npc { index } => {
            let npc_client_index = *index as u16;
            player.entity.face_target = Some(npc_client_index);
            player.player_info.add_mask(PlayerFaceEntityMask(npc_client_index));

            let player_client_index = player.index as u16 + 32768;
            let mut npc = world.npc_mut(*index);
            npc.stop();
            npc.entity.face_target = Some(player_client_index);
            if let Some(dir) = npc.position.direction_to(player.position) {
                npc.face_direction = dir;
            }
            npc.masks.add(NpcFaceMask(player_client_index));
        }
        InteractionTarget::Player { index } => {
            let client_index = *index as u16 + 32768;
            player.entity.face_target = Some(client_index);
            player.player_info.add_mask(PlayerFaceEntityMask(client_index));
        }
        InteractionTarget::Loc { .. } => {
            player
                .player_info
                .add_mask(crate::player::FaceDirectionMask(player.entity.face_direction));
        }
        InteractionTarget::Obj { .. } | InteractionTarget::ObjStack { .. } | InteractionTarget::Button { .. } => {}
    }
}

fn clear_face_if_needed(player: &mut Player, world: &World) {
    let Some(target_index) = player.entity.face_target.take() else {
        return;
    };

    player.player_info.add_mask(PlayerFaceEntityMask(65535));

    if target_index < 32768 {
        let npc_index = target_index as usize;
        if world.npcs.contains(npc_index) {
            let mut npc = world.npc_mut(npc_index);
            npc.entity.face_target = None;
            npc.masks.add(NpcFaceMask(65535));
        }
    }
}

#[player_system]
impl PlayerSystem for Interaction {
    type TickContext = ();

    fn create(_ctx: &PlayerInitContext) -> Self {
        Self { pending: None }
    }

    fn tick_context(_: &Arc<World>, _: &crate::player::PlayerSnapshot) {}
}
