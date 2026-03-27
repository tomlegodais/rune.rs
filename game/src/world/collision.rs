use crate::provider;
use crate::world::{Direction, Position};
use filesystem::{ArchiveId, Cache, IndexId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

const FLOOR_BLOCKED: u32 = 0x200000;
const FLOOR_DECO_BLOCKED: u32 = 0x40000;
const OBJ: u32 = 0x100;
const BLOCKED: u32 = FLOOR_BLOCKED | FLOOR_DECO_BLOCKED | OBJ;

const WALL_N: u32 = 0x2;
const WALL_E: u32 = 0x8;
const WALL_S: u32 = 0x20;
const WALL_W: u32 = 0x80;

const CORNER_NW: u32 = 0x1;
const CORNER_NE: u32 = 0x4;
const CORNER_SE: u32 = 0x10;
const CORNER_SW: u32 = 0x40;

const REGION_SIZE: usize = 64;
const PLANES: usize = 4;

type RegionKey = (u16, u16);
type TileFlags = [[[u32; REGION_SIZE]; REGION_SIZE]; PLANES];
type TileSettings = [[[u8; REGION_SIZE]; REGION_SIZE]; PLANES];

type WallClip = (u32, i32, i32, u32);
type CornerClip = (u32, i32, i32, u32, i32, i32, u32);
type SideCheck = (i32, i32, u32);

const WALL_STRAIGHT: [WallClip; 4] = [
    (WALL_W, -1, 0, WALL_E),
    (WALL_N, 0, 1, WALL_S),
    (WALL_E, 1, 0, WALL_W),
    (WALL_S, 0, -1, WALL_N),
];

const WALL_PILLAR: [WallClip; 4] = [
    (CORNER_NW, -1, 1, CORNER_SE),
    (CORNER_NE, 1, 1, CORNER_SW),
    (CORNER_SE, 1, -1, CORNER_NW),
    (CORNER_SW, -1, -1, CORNER_NE),
];

const WALL_L_CORNER: [CornerClip; 4] = [
    (WALL_N | WALL_W, -1, 0, WALL_E, 0, 1, WALL_S),
    (WALL_N | WALL_E, 0, 1, WALL_S, 1, 0, WALL_W),
    (WALL_S | WALL_E, 1, 0, WALL_W, 0, -1, WALL_N),
    (WALL_S | WALL_W, 0, -1, WALL_N, -1, 0, WALL_E),
];

const DEST_BLOCK: [u32; 8] = [
    BLOCKED | WALL_N | WALL_E | CORNER_NE,
    BLOCKED | WALL_N,
    BLOCKED | WALL_N | WALL_W | CORNER_NW,
    BLOCKED | WALL_E,
    BLOCKED | WALL_W,
    BLOCKED | WALL_E | WALL_S | CORNER_SE,
    BLOCKED | WALL_S,
    BLOCKED | WALL_S | WALL_W | CORNER_SW,
];

const DIAGONAL_SIDES: [[SideCheck; 2]; 8] = [
    [(-1, 0, BLOCKED | WALL_E), (0, -1, BLOCKED | WALL_N)],
    [(0, 0, 0), (0, 0, 0)],
    [(1, 0, BLOCKED | WALL_W), (0, -1, BLOCKED | WALL_N)],
    [(0, 0, 0), (0, 0, 0)],
    [(0, 0, 0), (0, 0, 0)],
    [(-1, 0, BLOCKED | WALL_E), (0, 1, BLOCKED | WALL_S)],
    [(0, 0, 0), (0, 0, 0)],
    [(1, 0, BLOCKED | WALL_W), (0, 1, BLOCKED | WALL_S)],
];

const OBJECT_SLOTS: [u8; 23] = [
    0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3,
];

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct GameObject {
    pub id: u32,
    pub loc_type: u8,
    pub rotation: u8,
}

type ObjectStore = [[[[Option<GameObject>; 4]; REGION_SIZE]; REGION_SIZE]; PLANES];

struct RegionData {
    flags: TileFlags,
    objects: Box<ObjectStore>,
}

pub struct CollisionMap {
    cache: Arc<Cache>,
    archive_index: HashMap<i32, ArchiveId>,
    regions: RwLock<HashMap<RegionKey, Arc<RegionData>>>,
}

impl CollisionMap {
    pub fn new(cache: Arc<Cache>) -> anyhow::Result<Self> {
        let mut archive_index = HashMap::new();
        let ref_table = cache.reference_table(IndexId::MAPS)?;
        for (archive_id, entry) in ref_table.iter_archives() {
            if let Some(hash) = entry.name_hash {
                archive_index.insert(hash, archive_id);
            }
        }

        Ok(CollisionMap {
            cache,
            archive_index,
            regions: RwLock::new(HashMap::new()),
        })
    }

    pub fn can_move(&self, from: Position, dir: Direction) -> bool {
        let to = from.step(dir);
        let di = dir as usize;

        if self.flag_at(to) & DEST_BLOCK[di] != 0 {
            return false;
        }

        let (dx, dy) = dir.delta();
        if dx != 0 && dy != 0 {
            for &(sx, sy, mask) in &DIAGONAL_SIDES[di] {
                if self.flag_at(Position::new(from.x + sx, from.y + sy, from.plane)) & mask != 0 {
                    return false;
                }
            }
        }

        true
    }

    pub fn flag_at(&self, pos: Position) -> u32 {
        let region = self.region_data(pos);
        let plane = pos.plane as usize;
        let lx = (pos.x & 63) as usize;
        let ly = (pos.y & 63) as usize;

        if plane < PLANES && lx < REGION_SIZE && ly < REGION_SIZE {
            region.flags[plane][lx][ly]
        } else {
            0
        }
    }

    pub fn get_object(&self, pos: Position, id: u32) -> Option<GameObject> {
        let region = self.region_data(pos);
        let plane = pos.plane as usize;
        let lx = (pos.x & 63) as usize;
        let ly = (pos.y & 63) as usize;

        if plane >= PLANES || lx >= REGION_SIZE || ly >= REGION_SIZE {
            return None;
        }

        region.objects[plane][lx][ly]
            .iter()
            .flatten()
            .find(|obj| obj.id == id)
            .cloned()
    }

    pub fn resolve_object_params(&self, pos: Position, id: u32) -> (i32, i32, u8) {
        let def = provider::get_loc_definition(id);
        let (base_w, base_h, base_access) = def
            .map(|d| (d.size_x as i32, d.size_y as i32, d.access_block_flag))
            .unwrap_or((1, 1, 0));

        let rotation = self
            .get_object(pos, id)
            .map(|obj| obj.rotation)
            .unwrap_or(0);

        let (w, h) = if rotation & 1 == 1 {
            (base_h, base_w)
        } else {
            (base_w, base_h)
        };

        let access = if rotation != 0 {
            ((base_access << rotation) & 0xF) | (base_access >> (4 - rotation))
        } else {
            base_access
        };

        (w, h, access)
    }

    fn region_data(&self, pos: Position) -> Arc<RegionData> {
        let rx = (pos.x >> 6) as u16;
        let ry = (pos.y >> 6) as u16;
        let key = (rx, ry);

        if let Some(data) = self.regions.read().unwrap().get(&key) {
            return Arc::clone(data);
        }

        let data = Arc::new(self.load_region(rx, ry));
        self.regions.write().unwrap().insert(key, Arc::clone(&data));
        data
    }

    fn load_region(&self, rx: u16, ry: u16) -> RegionData {
        let mut flags = [[[0u32; REGION_SIZE]; REGION_SIZE]; PLANES];
        let mut settings = [[[0u8; REGION_SIZE]; REGION_SIZE]; PLANES];

        const NONE: Option<GameObject> = None;
        let mut objects = Box::new([[[[NONE; 4]; REGION_SIZE]; REGION_SIZE]; PLANES]);

        let map_hash = filesystem::name_hash(&format!("m{}_{}", rx, ry));
        let loc_hash = filesystem::name_hash(&format!("l{}_{}", rx, ry));

        if let Some(&archive_id) = self.archive_index.get(&map_hash)
            && let Ok(data) = self.cache.read_archive(IndexId::MAPS, archive_id)
        {
            parse_tile_settings(&data, &mut flags, &mut settings);
        }

        if let Some(&archive_id) = self.archive_index.get(&loc_hash)
            && let Ok(data) = self.cache.read_archive(IndexId::MAPS, archive_id)
        {
            parse_loc_placements(&data, &mut flags, &settings, &mut objects);
        }

        RegionData { flags, objects }
    }
}

fn parse_tile_settings(data: &[u8], flags: &mut TileFlags, settings: &mut TileSettings) {
    let mut buf = Bytes::copy_from_slice(data);

    for plane_settings in settings.iter_mut() {
        for col in plane_settings.iter_mut() {
            for cell in col.iter_mut() {
                loop {
                    if !buf.has_remaining() {
                        return;
                    }

                    let opcode = buf.get_u8() as u16;
                    if opcode == 0 {
                        break;
                    } else if opcode == 1 {
                        buf.advance(1);
                        break;
                    } else if opcode <= 49 {
                        buf.advance(1);
                    } else if opcode <= 81 {
                        *cell = (opcode - 49) as u8;
                    }
                }
            }
        }
    }

    for (plane, plane_settings) in settings.iter().enumerate() {
        for (x, col) in plane_settings.iter().enumerate() {
            for (y, &cell) in col.iter().enumerate() {
                if cell & 0x1 == 0 {
                    continue;
                }

                let ep = if settings[1][x][y] & 0x2 != 0 {
                    plane.wrapping_sub(1)
                } else {
                    plane
                };

                if ep < PLANES {
                    flags[ep][x][y] |= FLOOR_BLOCKED;
                }
            }
        }
    }
}

fn parse_loc_placements(
    data: &[u8],
    flags: &mut TileFlags,
    settings: &TileSettings,
    objects: &mut ObjectStore,
) {
    let mut buf = Bytes::copy_from_slice(data);
    let mut loc_id: i32 = -1;

    loop {
        if !buf.has_remaining() {
            break;
        }
        let delta = buf.get_extended_smart();
        if delta == 0 {
            break;
        }
        loc_id = loc_id.wrapping_add(delta as i32);

        let def = provider::get_loc_definition(loc_id as u32);

        let mut packed_pos: u32 = 0;
        loop {
            if !buf.has_remaining() {
                break;
            }
            let pos_delta = buf.get_smart() as u32;
            if pos_delta == 0 {
                break;
            }
            packed_pos = packed_pos.wrapping_add(pos_delta - 1);

            let local_y = (packed_pos & 0x3F) as usize;
            let local_x = ((packed_pos >> 6) & 0x3F) as usize;
            let raw_plane = (packed_pos >> 12) as usize;

            let attributes = buf.get_u8();
            let loc_type = attributes >> 2;
            let rotation = attributes & 0x3;

            if local_x >= REGION_SIZE || local_y >= REGION_SIZE || raw_plane >= PLANES {
                continue;
            }

            let plane = if settings[1][local_x][local_y] & 0x2 != 0 {
                raw_plane.wrapping_sub(1)
            } else {
                raw_plane
            };
            if plane >= PLANES {
                continue;
            }

            if (loc_type as usize) < OBJECT_SLOTS.len() {
                let slot = OBJECT_SLOTS[loc_type as usize] as usize;
                objects[plane][local_x][local_y][slot] = Some(GameObject {
                    id: loc_id as u32,
                    loc_type,
                    rotation,
                });
            }

            let Some(def) = def else {
                continue;
            };

            match loc_type {
                0..=3 if def.block_walk => {
                    add_wall(flags, plane, local_x, local_y, loc_type, rotation);
                }
                9..=21 if def.block_walk => {
                    let (sx, sy) = if rotation & 1 == 1 {
                        (def.size_y as usize, def.size_x as usize)
                    } else {
                        (def.size_x as usize, def.size_y as usize)
                    };
                    add_object(flags, plane, local_x, local_y, sx, sy);
                }
                22 if def.solid == 1 => {
                    add_floor_deco(flags, plane, local_x, local_y);
                }
                _ => {}
            }
        }
    }
}

fn set_flag(flags: &mut TileFlags, plane: usize, x: i32, y: i32, flag: u32) {
    let (ux, uy) = (x as usize, y as usize);
    if ux < REGION_SIZE && uy < REGION_SIZE {
        flags[plane][ux][uy] |= flag;
    }
}

fn add_wall(flags: &mut TileFlags, plane: usize, x: usize, y: usize, loc_type: u8, rotation: u8) {
    if x >= REGION_SIZE || y >= REGION_SIZE {
        return;
    }
    let (ix, iy) = (x as i32, y as i32);
    let r = rotation as usize;

    match loc_type {
        0 => {
            let (sf, dx, dy, nf) = WALL_STRAIGHT[r];
            flags[plane][x][y] |= sf;
            set_flag(flags, plane, ix + dx, iy + dy, nf);
        }
        1 | 3 => {
            let (sf, dx, dy, nf) = WALL_PILLAR[r];
            flags[plane][x][y] |= sf;
            set_flag(flags, plane, ix + dx, iy + dy, nf);
        }
        2 => {
            let (sf, dx1, dy1, nf1, dx2, dy2, nf2) = WALL_L_CORNER[r];
            flags[plane][x][y] |= sf;
            set_flag(flags, plane, ix + dx1, iy + dy1, nf1);
            set_flag(flags, plane, ix + dx2, iy + dy2, nf2);
        }
        _ => {}
    }
}

fn add_object(flags: &mut TileFlags, plane: usize, lx: usize, ly: usize, sx: usize, sy: usize) {
    for dx in 0..sx {
        for dy in 0..sy {
            let (x, y) = (lx + dx, ly + dy);
            if x < REGION_SIZE && y < REGION_SIZE {
                flags[plane][x][y] |= OBJ;
            }
        }
    }
}

fn add_floor_deco(flags: &mut TileFlags, plane: usize, x: usize, y: usize) {
    if x < REGION_SIZE && y < REGION_SIZE {
        flags[plane][x][y] |= FLOOR_DECO_BLOCKED;
    }
}
