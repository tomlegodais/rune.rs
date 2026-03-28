use crate::entity::Mask;
use crate::entity::MoveStep;
use crate::player::PlayerSnapshot;
use crate::player::state::{MAX_PLAYERS, PlayerState};
use crate::world::{Position, Teleport};
use net::Outbox;
use std::array;
use std::ops::{Index, IndexMut};

pub struct PlayerInfo {
    outbox: Outbox,

    pub self_index: usize,
    pub players: [PlayerState; MAX_PLAYERS],
    pub pending_add: Vec<PlayerSnapshot>,
    pub pending_remove: Vec<usize>,
}

impl PlayerInfo {
    pub fn new(
        outbox: Outbox,
        self_index: usize,
        snapshots: &[PlayerSnapshot],
        initial_masks: &[&dyn Mask],
    ) -> Self {
        let mut players: [PlayerState; MAX_PLAYERS] = array::from_fn(|_| PlayerState::default());
        players[self_index].local = true;
        players[self_index].masks.extend(initial_masks);

        for s in snapshots {
            if s.index != self_index {
                players[s.index].region_hash = s.position.region_hash();
            }
        }

        Self {
            outbox,
            self_index,
            players,
            pending_add: Vec::new(),
            pending_remove: Vec::new(),
        }
    }

    pub fn sync(&mut self, others: &[PlayerSnapshot], is_within_view: impl Fn(Position) -> bool) {
        for other in others {
            if other.index == self.self_index {
                continue;
            }

            self.players[other.index].teleport = other.teleport;
            self.players[other.index].move_step = other.move_step;
            self.players[other.index].masks = other.masks.clone();

            let is_local = self.players[other.index].local;
            let in_range = is_within_view(other.position);

            if in_range && !is_local {
                self.pending_add.push(other.clone());
            } else if !in_range && is_local {
                self.pending_remove.push(other.index);
            }
        }

        for idx in 1..MAX_PLAYERS {
            if idx == self.self_index || !self.players[idx].local {
                continue;
            }
            if !others.iter().any(|s| s.index == idx) {
                self.pending_remove.push(idx);
            }
        }
    }

    pub async fn flush(&mut self) {
        let frame = crate::player::gpi::encode(self);
        let _ = self.outbox.send(frame).await;
    }

    pub fn add_mask(&mut self, mask: impl Mask) {
        self.players[self.self_index].masks.add(mask);
    }

    pub fn teleport(&mut self, teleport: Teleport) {
        self.players[self.self_index].teleport = Some(teleport);
    }

    pub fn set_move_step(&mut self, step: MoveStep) {
        self.players[self.self_index].move_step = step;
    }

    pub fn reset(&mut self) {
        for pending in self.pending_add.drain(..) {
            self.players[pending.index].local = true;
        }
        for index in self.pending_remove.drain(..) {
            self.players[index].local = false;
        }
        for i in 1..MAX_PLAYERS {
            self.players[i].activity >>= 1;
            self.players[i].teleport = None;
            self.players[i].move_step = MoveStep::None;
            self.players[i].masks.clear();
        }
    }

    pub fn self_state(&self) -> &PlayerState {
        &self.players[self.self_index]
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
