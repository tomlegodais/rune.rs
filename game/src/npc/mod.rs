mod gni;
mod info;
mod mask;

use std::ops::{Deref, DerefMut};

pub use gni::encode_npc_info;
pub use info::NpcInfo;
pub use mask::{FaceEntityMask, SeqMask, SpotAnim1Mask, SpotAnim2Mask};
use rand::Rng;
use strum::IntoEnumIterator;

use crate::{
    entity::{Entity, Hit, MaskBlock, MoveStep, Seq, SeqBuilder, SpotAnim, SpotAnimBuilder},
    provider,
    world::{Direction, Position, Teleport},
};

const DEATH_SEQ: u16 = 836;
const DEATH_TICKS: u16 = 5;

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

pub struct Npc {
    pub entity: Entity,
    pub npc_id: u16,
    pub spawn_position: Position,
    pub wander_radius: u8,
    pub running: bool,
    pub move_step: MoveStep,
    pub teleport: Option<Teleport>,
    pub masks: MaskBlock,
    pub current_hp: u32,
    pub max_hp: u32,
    death_timer: Option<u16>,
}

impl Npc {
    pub fn new(index: usize, npc_id: u16, position: Position, wander_radius: u8, max_hp: u32) -> Self {
        Self {
            npc_id,
            spawn_position: position,
            wander_radius,
            entity: Entity::new(index, position),
            move_step: MoveStep::None,
            masks: MaskBlock::new(&mask::NPC_MASKS),
            teleport: None,
            running: false,
            current_hp: max_hp,
            max_hp,
            death_timer: None,
        }
    }

    pub fn is_dying(&self) -> bool {
        self.death_timer.is_some()
    }

    pub fn is_dead(&self) -> bool {
        matches!(self.death_timer, Some(0))
    }

    pub fn damage(&mut self, mut hit: Hit) -> bool {
        if self.death_timer.is_some() {
            return false;
        }
        hit.damage = hit.damage.min(self.current_hp as u16);
        self.current_hp -= hit.damage as u32;
        hit.hp_ratio = (self.current_hp * 255 / self.max_hp) as u8;
        self.add_hit(hit);
        let dead = self.current_hp == 0;
        if dead {
            self.on_death();
        }
        dead
    }

    fn add_hit(&mut self, hit: Hit) {
        if self.masks.has(mask::NpcMask::HIT_1) {
            self.masks.add(mask::Hit2Mask(hit));
        } else {
            self.masks.add(mask::Hit1Mask(hit));
        }
    }

    fn on_death(&mut self) {
        self.death_timer = Some(DEATH_TICKS);
        self.entity.stop();
        self.seq(DEATH_SEQ);
    }

    pub fn tick_death(&mut self) -> bool {
        let Some(ref mut timer) = self.death_timer else {
            return false;
        };
        *timer = timer.saturating_sub(1);
        true
    }

    pub fn wander(&mut self) {
        if self.wander_radius == 0 || self.has_steps() || self.face_target.is_some() {
            return;
        }

        let mut rng = rand::rng();
        if !rng.random_ratio(1, 8) {
            return;
        }

        let radius = self.wander_radius as i32;
        let dx = self.position.x - self.spawn_position.x;
        let dy = self.position.y - self.spawn_position.y;
        let weighted: Vec<(Direction, u32)> = Direction::iter()
            .map(|d| {
                let (ddx, ddy) = d.delta();
                let closer_x = ddx.signum() == -dx.signum() || dx == 0;
                let closer_y = ddy.signum() == -dy.signum() || dy == 0;
                let weight = if closer_x && closer_y { 3 } else { 1 };
                (d, weight)
            })
            .collect();

        let total: u32 = weighted.iter().map(|(_, w)| w).sum();
        let mut pick = rng.random_range(0..total);
        let dir = weighted
            .into_iter()
            .find(|(_, w)| {
                let hit = pick < *w;
                pick = pick.saturating_sub(*w);
                hit
            })
            .map(|(d, _)| d)
            .unwrap();

        let candidate = self.position.step(dir);
        let in_bounds = (candidate.x - self.spawn_position.x).abs() <= radius
            && (candidate.y - self.spawn_position.y).abs() <= radius;

        if in_bounds && provider::get_collision().can_move(self.position, dir) {
            self.walk_queue.push_back(candidate);
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

    pub fn seq(&mut self, id: u16) -> SeqBuilder<impl FnOnce(Seq) + '_> {
        SeqBuilder::new(id, |a| self.masks.add(SeqMask(a)))
    }

    pub fn spot_anim(&mut self, id: u16) -> SpotAnimBuilder<impl FnOnce(SpotAnim) + '_> {
        SpotAnimBuilder::new(id, |sa| {
            if self.masks.has(mask::NpcMask::SPOT_ANIM_1) {
                self.masks.add(SpotAnim2Mask(sa));
            } else {
                self.masks.add(SpotAnim1Mask(sa));
            }
        })
    }

    pub fn force_talk(&mut self, message: String) {
        self.masks.add(mask::ForceTalkMask(message));
    }
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
