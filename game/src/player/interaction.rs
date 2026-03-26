use crate::npc::FaceEntityMask as NpcFaceMask;
use crate::player::Player;
use crate::player::action::{ActionShared, ActionState};
use crate::player::mask::FaceEntityMask as PlayerFaceEntityMask;
use crate::player::system::{PlayerInitContext, PlayerSystem};
use crate::world::{Position, World};
use macros::player_system;
use net::ClickOption;
use std::sync::Arc;

pub struct Interaction {
    pending: Option<PendingInteraction>,
}

pub struct PendingInteraction {
    pub target: InteractionTarget,
    pub option: ClickOption,
}

pub enum InteractionTarget {
    Object { id: u16, x: i32, y: i32 },
    Npc { index: usize },
    Player { index: usize },
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
            Self::Object { x, y, .. } => Some(Position::new(*x, *y, plane)),
            Self::Npc { index } => world
                .npcs
                .contains(*index)
                .then(|| world.npc(*index).position),
            Self::Player { index } => world
                .players
                .contains(*index)
                .then(|| world.player(*index).position),
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

    let Some(pending) = player.system::<Interaction>().pending() else {
        clear_face_if_needed(player, world);
        return;
    };

    let Some(target_pos) = pending.target.target_position(world, player.position.plane) else {
        player.systems.get_mut::<Interaction>().clear();
        return;
    };

    if !adjacent(player.position, target_pos) {
        if !player.entity.has_steps() {
            player.systems.get_mut::<Interaction>().clear();
        }
        return;
    }

    player.entity.stop();
    let pending = player.systems.get_mut::<Interaction>().take().unwrap();
    face_target(player, world, &pending.target, target_pos);

    if let Some(future) = crate::handler::dispatch(player, pending.target, pending.option) {
        let shared = Arc::new(ActionShared::new());
        crate::player::action::set_action_context(player as *mut Player, shared.clone());

        let mut action_state = ActionState {
            active: future,
            shared: shared.clone(),
        };

        let poll_result = crate::player::action::poll_action(&mut action_state);
        crate::player::action::clear_action_context();

        if poll_result.is_pending() {
            world.action_states.lock().insert(player_index, action_state);
        } else {
            clear_face_if_needed(player, world);
        }
    }
}

fn adjacent(a: Position, b: Position) -> bool {
    a.chebyshev_pos(b) == 1
}

fn face_target(
    player: &mut Player,
    world: &World,
    target: &InteractionTarget,
    target_pos: Position,
) {
    if let Some(dir) = player.position.direction_to(target_pos) {
        player.entity.face_direction = dir;
    }

    match target {
        InteractionTarget::Npc { index } => {
            let npc_client_index = *index as u16;
            player.entity.face_target = Some(npc_client_index);
            player
                .player_info
                .add_mask(PlayerFaceEntityMask(npc_client_index));

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
            player
                .player_info
                .add_mask(PlayerFaceEntityMask(client_index));
        }
        InteractionTarget::Object { .. } => {
            player
                .player_info
                .add_mask(crate::player::FaceDirectionMask(
                    player.entity.face_direction,
                ));
        }
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
    fn create(_ctx: &PlayerInitContext) -> Self {
        Self { pending: None }
    }
}
