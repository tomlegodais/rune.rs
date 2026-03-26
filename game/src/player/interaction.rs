use crate::npc::FaceEntityMask as NpcFaceMask;
use crate::player::Player;
use crate::player::mask::FaceEntityMask as PlayerFaceEntityMask;
use crate::player::system::{PlayerInitContext, PlayerSystem};
use crate::world::{Position, World};
use macros::player_system;
use net::ClickOption;

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

pub async fn resolve(player: &mut Player, world: &World) {
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
    crate::handler::dispatch(player, &pending.target, pending.option).await;
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
