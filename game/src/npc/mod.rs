pub(crate) mod gni;
mod info;
mod mask;

use crate::entity::Entity;
use crate::entity::MaskBlock;
use crate::entity::MoveStep;
use crate::world::{Direction, Position, Teleport};
use std::ops::{Deref, DerefMut};

pub(crate) use info::NpcInfo;

pub struct Npc {
    pub entity: Entity,
    pub npc_id: u16,
    pub running: bool,
    pub move_step: MoveStep,
    pub teleport: Option<Teleport>,
    pub masks: MaskBlock,
}

impl Deref for Npc {
    type Target = Entity;
    fn deref(&self) -> &Entity {
        &self.entity
    }
}

impl DerefMut for Npc {
    fn deref_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl Npc {
    pub fn new(index: usize, npc_id: u16, position: Position) -> Self {
        Self {
            npc_id,
            entity: Entity::new(index, position),
            move_step: MoveStep::None,
            masks: MaskBlock::new(&mask::NPC_MASKS),
            teleport: None,
            running: false,
        }
    }

    pub fn snapshot(&self) -> NpcSnapshot {
        NpcSnapshot {
            index: self.index,
            npc_id: self.npc_id,
            position: self.position,
            face_direction: self.face_direction,
            masks: self.masks.clone(),
            teleport: self.teleport,
            move_step: self.move_step,
            running: self.running,
        }
    }

    pub fn process_movement(&mut self) {
        let Some(walk_dir) = self.entity.step() else {
            return;
        };

        self.move_step = MoveStep::Walk(walk_dir);
    }

    pub fn reset(&mut self) {
        self.teleport = None;
        self.move_step = MoveStep::None;
        self.masks.clear();
    }

    pub fn force_talk(&mut self, message: String) {
        self.masks.add(mask::ForceTalkMask(message));
    }
}

#[derive(Clone)]
pub struct NpcSnapshot {
    pub index: usize,
    pub npc_id: u16,
    pub position: Position,
    pub face_direction: Direction,
    pub masks: MaskBlock,
    pub teleport: Option<Teleport>,
    pub move_step: MoveStep,
    pub running: bool,
}
