mod anim;
mod mask;
mod movement;
mod spotanim;

use std::{
    collections::VecDeque,
    sync::{Arc, Weak},
};

pub(crate) use anim::{Anim, AnimBuilder};
pub(crate) use mask::{Mask, MaskBlock, MaskConfig, MaskFlags};
pub(crate) use spotanim::{SpotAnim, SpotAnimBuilder};

use crate::world::{Direction, Position, World};

#[derive(Copy, Clone, Default)]
pub enum MoveStep {
    #[default]
    None,
    Walk(Direction),
    Run(u8),
}

pub struct Entity {
    world: Weak<World>,

    pub index: usize,
    pub position: Position,
    pub face_direction: Direction,
    pub walk_queue: VecDeque<Position>,
    pub face_target: Option<u16>,
}

impl Entity {
    pub fn new(index: usize, position: Position) -> Self {
        Self {
            world: Weak::new(),
            index,
            position,
            face_direction: Direction::South,
            walk_queue: VecDeque::new(),
            face_target: None,
        }
    }

    pub fn world(&self) -> Arc<World> {
        self.world.upgrade().expect("world has been dropped")
    }

    pub(crate) fn set_world(&mut self, world: &Arc<World>) {
        self.world = Arc::downgrade(world);
    }
}
