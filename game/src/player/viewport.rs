use crate::player::{Appearance, PlayerSnapshot};
use crate::world::{Position, RegionId};

const VIEW_DISTANCES: [i32; 4] = [104, 120, 136, 168];
const MAX_PLAYERS: usize = 2048;

#[derive(Copy, Clone)]
pub struct PlayerState {
    pub local: bool,
    pub activity: u8,
    pub region_hash: u32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            local: false,
            activity: 0,
            region_hash: 0,
        }
    }
}

pub struct Viewport {
    pub view_distance: usize,
    pub region_base: Position,

    pub players: [PlayerState; MAX_PLAYERS],
    pub pending_add: Vec<PlayerSnapshot>,
    pub pending_remove: Vec<usize>,
}

impl Viewport {
    pub fn new(
        self_id: usize,
        position: Position,
        view_distance: usize,
        snapshots: &[PlayerSnapshot],
    ) -> Self {
        let half_chunks = VIEW_DISTANCES[view_distance] >> 4;
        let mut players = [PlayerState::default(); MAX_PLAYERS];
        players[self_id].local = true;

        for s in snapshots {
            if s.id != self_id {
                players[s.id].region_hash = s.position.region_hash();
            }
        }

        Self {
            view_distance,
            region_base: Position::from_chunks(
                position.chunk_x() - half_chunks,
                position.chunk_y() - half_chunks,
            ),
            players,
            pending_add: Vec::new(),
            pending_remove: Vec::new(),
        }
    }

    pub fn needs_rebuild(&self, position: Position) -> bool {
        let local_x = position.x - self.region_base.x;
        let local_y = position.y - self.region_base.y;

        let map_size = VIEW_DISTANCES[self.view_distance];
        let lower = map_size >> 4;
        let upper = map_size - lower;

        local_x < lower || local_x >= upper || local_y < lower || local_y >= upper
    }

    pub fn rebuild(&mut self, position: Position) {
        let half_chunks = VIEW_DISTANCES[self.view_distance] >> 4;
        self.region_base = Position::from_chunks(
            position.chunk_x() - half_chunks,
            position.chunk_y() - half_chunks,
        );
    }

    pub fn sync(&mut self, self_id: usize, others: &[PlayerSnapshot]) {
        for other in others {
            if other.id == self_id {
                continue;
            }

            let is_local = self.players[other.id].local;
            let in_range = self.is_within_view(other.position);

            if in_range && !is_local {
                self.pending_add.push(other.clone());
            } else if !in_range && is_local {
                self.pending_remove.push(other.id);
            }
        }
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
        }
    }

    pub fn is_within_view(&self, other: Position) -> bool {
        let map_size = VIEW_DISTANCES[self.view_distance];
        let dx = (other.x - self.region_base.x).abs();
        let dy = (other.y - self.region_base.y).abs();
        dx <= map_size && dy <= map_size
    }

    pub fn region_ids(&self) -> Vec<RegionId> {
        let map_size = VIEW_DISTANCES[self.view_distance];
        let max = Position::new(
            self.region_base.x + map_size,
            self.region_base.y + map_size,
            0,
        );
        self.region_base.region_id().to(max.region_id()).collect()
    }
}
