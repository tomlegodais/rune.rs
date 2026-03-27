use crate::world::{Position, RegionId};

const VIEW_DISTANCES: [i32; 4] = [104, 120, 136, 168];

pub struct Viewport {
    pub view_distance: usize,
    pub region_base: Position,
}

impl Viewport {
    pub fn new(position: Position, view_distance: usize) -> Self {
        let half_chunks = VIEW_DISTANCES[view_distance] >> 4;

        Self {
            view_distance,
            region_base: Position::from_chunks(
                position.chunk_x() - half_chunks,
                position.chunk_y() - half_chunks,
            ),
        }
    }

    pub fn needs_rebuild(&self, position: Position) -> bool {
        let half_chunks = VIEW_DISTANCES[self.view_distance] >> 4;
        let center_cx = self.region_base.chunk_x() + half_chunks;
        let center_cy = self.region_base.chunk_y() + half_chunks;
        let threshold = ((VIEW_DISTANCES[self.view_distance] >> 3) / 2) - 1;

        (position.chunk_x() - center_cx).abs() >= threshold
            || (position.chunk_y() - center_cy).abs() >= threshold
    }

    pub fn rebuild(&mut self, position: Position) {
        let half_chunks = VIEW_DISTANCES[self.view_distance] >> 4;
        self.region_base = Position::from_chunks(
            position.chunk_x() - half_chunks,
            position.chunk_y() - half_chunks,
        );
    }

    pub fn is_within_view(&self, pos: Position, other: Position) -> bool {
        other.plane == pos.plane && (other.x - pos.x).abs() <= 15 && (other.y - pos.y).abs() <= 15
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
