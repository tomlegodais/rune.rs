use crate::player::Mask;
use crate::player::PlayerSnapshot;
use crate::player::state::{MAX_PLAYERS, MoveStep, PlayerState};
use crate::world::{Position, Teleport};
use std::array;
use std::ops::{Index, IndexMut};

pub struct PlayerInfo {
    pub self_id: usize,
    pub players: [PlayerState; MAX_PLAYERS],
    pub pending_add: Vec<PlayerSnapshot>,
    pub pending_remove: Vec<usize>,
}

impl PlayerInfo {
    pub fn new(self_id: usize, snapshots: &[PlayerSnapshot], initial_masks: &[&dyn Mask]) -> Self {
        let mut players: [PlayerState; MAX_PLAYERS] = array::from_fn(|_| PlayerState::default());
        players[self_id].local = true;
        players[self_id].masks.extend(initial_masks);

        for s in snapshots {
            if s.id != self_id {
                players[s.id].region_hash = s.position.region_hash();
            }
        }

        Self {
            self_id,
            players,
            pending_add: Vec::new(),
            pending_remove: Vec::new(),
        }
    }

    pub fn sync(&mut self, others: &[PlayerSnapshot], is_within_view: impl Fn(Position) -> bool) {
        for other in others {
            if other.id == self.self_id {
                continue;
            }

            self.players[other.id].teleport = other.teleport;
            self.players[other.id].move_step = other.move_step;
            self.players[other.id].masks = other.masks.clone();

            let is_local = self.players[other.id].local;
            let in_range = is_within_view(other.position);

            if in_range && !is_local {
                self.pending_add.push(other.clone());
            } else if !in_range && is_local {
                self.pending_remove.push(other.id);
            }
        }

        for idx in 1..MAX_PLAYERS {
            if idx == self.self_id || !self.players[idx].local {
                continue;
            }
            if !others.iter().any(|s| s.id == idx) {
                self.pending_remove.push(idx);
            }
        }
    }

    pub fn add_mask(&mut self, mask: impl Mask) {
        self.players[self.self_id].masks.add(mask);
    }

    pub fn teleport(&mut self, teleport: Teleport) {
        self.players[self.self_id].teleport = Some(teleport);
    }

    pub fn set_move_step(&mut self, step: MoveStep) {
        self.players[self.self_id].move_step = step;
    }

    pub fn reset(&mut self) {
        for pending in self.pending_add.drain(..) {
            self.players[pending.id].local = true;
        }
        for id in self.pending_remove.drain(..) {
            self.players[id].local = false;
        }
        for i in 1..MAX_PLAYERS {
            self.players[i].activity >>= 1;
            self.players[i].teleport = None;
            self.players[i].move_step = MoveStep::None;
            self.players[i].masks.clear();
        }
    }

    pub fn self_state(&self) -> &PlayerState {
        &self.players[self.self_id]
    }
}

impl Index<usize> for PlayerInfo {
    type Output = PlayerState;

    fn index(&self, idx: usize) -> &PlayerState {
        &self.players[idx]
    }
}

impl IndexMut<usize> for PlayerInfo {
    fn index_mut(&mut self, idx: usize) -> &mut PlayerState {
        &mut self.players[idx]
    }
}
