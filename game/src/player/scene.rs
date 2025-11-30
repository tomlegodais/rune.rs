use crate::world::Position;

const SCENE_SIZES: [u16; 4] = [104, 120, 136, 168];

pub struct Scene {
    pub size: u8,
    pub center_chunk_x: i32,
    pub center_chunk_y: i32,
    pub region_ids: Vec<u16>,
}

impl Scene {
    pub fn new(position: Position, size: u8) -> Self {
        let chunk_x = position.chunk_x();
        let chunk_y = position.chunk_y();
        let region_ids = calc_region_ids(chunk_x, chunk_y, size);

        Self {
            size,
            center_chunk_x: chunk_x,
            center_chunk_y: chunk_y,
            region_ids,
        }
    }

    pub fn needs_rebuild(&self, position: Position) -> bool {
        let new_chunk_x = position.chunk_x();
        let new_chunk_y = position.chunk_y();
        window_changed(
            self.center_chunk_x,
            self.center_chunk_y,
            new_chunk_x,
            new_chunk_y,
            self.size,
        )
    }

    pub fn rebuild(&mut self, position: Position) {
        self.center_chunk_x = position.chunk_x();
        self.center_chunk_y = position.chunk_y();
        self.region_ids = calc_region_ids(self.center_chunk_x, self.center_chunk_y, self.size);
    }
}

fn calc_region_ids(chunk_x: i32, chunk_y: i32, map_size: u8) -> Vec<u16> {
    let scene_hash = (SCENE_SIZES[map_size as usize] as i32) >> 4;
    let (min_region_x, min_region_y) = ((chunk_x - scene_hash) / 8, (chunk_y - scene_hash) / 8);
    let (start_x, start_y) = (min_region_x.max(0), min_region_y.max(0));
    let (end_x, end_y) = ((chunk_x + scene_hash) / 8, (chunk_y + scene_hash) / 8);
    let mut region_ids = Vec::new();
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            let region_id = (y + (x << 8)) as u16;
            region_ids.push(region_id);
        }
    }

    region_ids
}

fn region_bounds(chunk: i32, map_hash: i32) -> (i32, i32) {
    let min_r = (chunk - map_hash) / 8;
    let max_r = (chunk + map_hash) / 8;
    (min_r, max_r)
}

fn window_changed(
    old_chunk_x: i32,
    old_chunk_y: i32,
    new_chunk_x: i32,
    new_chunk_y: i32,
    map_size: u8,
) -> bool {
    let scene_hash = (SCENE_SIZES[map_size as usize] as i32) >> 4;

    let (old_min_rx, old_max_rx) = region_bounds(old_chunk_x, scene_hash);
    let (old_min_ry, old_max_ry) = region_bounds(old_chunk_y, scene_hash);

    let (new_min_rx, new_max_rx) = region_bounds(new_chunk_x, scene_hash);
    let (new_min_ry, new_max_ry) = region_bounds(new_chunk_y, scene_hash);

    old_min_rx != new_min_rx
        || old_max_rx != new_max_rx
        || old_min_ry != new_min_ry
        || old_max_ry != new_max_ry
}
