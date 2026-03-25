mod mask;
mod movement;

pub(crate) use mask::{Mask, MaskBlock, MaskConfig, MaskFlags};

use crate::world::{Direction, Position, RegionId, World};
use std::collections::VecDeque;

#[derive(Copy, Clone, Default)]
pub enum MoveStep {
    #[default]
    None,
    Walk(Direction),
    Run(u8),
}

pub struct Entity {
    world: *const World,

    pub index: usize,
    pub position: Position,
    pub current_region: RegionId,
    pub face_direction: Direction,
    pub walk_queue: VecDeque<Position>,
}

impl Entity {
    pub fn new(index: usize, position: Position) -> Self {
        Self {
            world: std::ptr::null(),
            index,
            current_region: position.region_id(),
            position,
            face_direction: Direction::South,
            walk_queue: VecDeque::new(),
        }
    }

    pub fn world(&self) -> &World {
        unsafe { &*self.world }
    }

    pub(crate) fn set_world(&mut self, world: &World) {
        self.world = world as *const World;
    }
}

unsafe impl Send for Entity {}
unsafe impl Sync for Entity {}
