use crate::world::RegionId;

#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub plane: u16,
}

impl Position {
    pub fn new(x: u16, y: u16, plane: u16) -> Self {
        Self { x, y, plane }
    }

    pub fn chunk_x(self) -> i32 {
        (self.x as i32) >> 3
    }

    pub fn chunk_y(self) -> i32 {
        (self.y as i32) >> 3
    }

    pub fn region_id(self) -> RegionId {
        let rx = self.x >> 6;
        let ry = self.y >> 6;
        RegionId::from_coords(rx, ry)
    }

    pub fn bits(self) -> u32 {
        ((self.plane as u32 & 0x3) << 28)
            | ((self.x as u32 & 0x3fff) << 14)
            | (self.y as u32 & 0x3fff)
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new(3093, 3493, 0)
    }
}
