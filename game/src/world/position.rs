use crate::world::RegionId;

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub plane: i32,
}

impl Position {
    pub fn new(x: i32, y: i32, plane: i32) -> Self {
        Self { x, y, plane }
    }

    pub fn chunk_x(self) -> i32 {
        self.x >> 3
    }

    pub fn chunk_y(self) -> i32 {
        self.y >> 3
    }

    pub fn from_chunks(chunk_x: i32, chunk_y: i32) -> Self {
        Self {
            x: chunk_x << 3,
            y: chunk_y << 3,
            plane: 0,
        }
    }

    pub fn region_id(self) -> RegionId {
        let rx = (self.x >> 6) as u16;
        let ry = (self.y >> 6) as u16;
        RegionId::from_coords(rx, ry)
    }

    pub fn region_hash(self) -> u32 {
        let rx = (self.x >> 6) as u32;
        let ry = (self.y >> 6) as u32;
        ry + (rx << 8) + ((self.plane as u32 & 0x3) << 16)
    }

    pub fn to_bits(self) -> u32 {
        ((self.plane as u32 & 0x3) << 28)
            | ((self.x as u32 & 0x3fff) << 14)
            | (self.y as u32 & 0x3fff)
    }
}

#[derive(Copy, Clone)]
pub struct Teleport {
    pub from: Position,
    pub to: Position,
}

impl Default for Position {
    fn default() -> Self {
        Position::new(3093, 3493, 0)
    }
}
