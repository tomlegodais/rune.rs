use crate::world::RegionId;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    pub fn direction_to(self, other: Position) -> Option<Direction> {
        let dx = (other.x - self.x).signum();
        let dy = (other.y - self.y).signum();
        Direction::from_delta(dx, dy)
    }

    pub fn step(self, dir: Direction) -> Position {
        let (dx, dy) = match dir {
            Direction::SouthWest => (-1, -1),
            Direction::South => (0, -1),
            Direction::SouthEast => (1, -1),
            Direction::West => (-1, 0),
            Direction::East => (1, 0),
            Direction::NorthWest => (-1, 1),
            Direction::North => (0, 1),
            Direction::NorthEast => (1, 1),
        };
        Position::new(self.x + dx, self.y + dy, self.plane)
    }

    pub fn to_bits(self) -> u32 {
        ((self.plane as u32 & 0x3) << 28)
            | ((self.x as u32 & 0x3fff) << 14)
            | (self.y as u32 & 0x3fff)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    SouthWest = 0,
    South = 1,
    SouthEast = 2,
    West = 3,
    East = 4,
    NorthWest = 5,
    North = 6,
    NorthEast = 7,
}

impl Direction {
    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        match (dx, dy) {
            (-1, -1) => Some(Self::SouthWest),
            (0, -1) => Some(Self::South),
            (1, -1) => Some(Self::SouthEast),
            (-1, 0) => Some(Self::West),
            (1, 0) => Some(Self::East),
            (-1, 1) => Some(Self::NorthWest),
            (0, 1) => Some(Self::North),
            (1, 1) => Some(Self::NorthEast),
            _ => None,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::SouthWest => (-1, -1),
            Self::South => (0, -1),
            Self::SouthEast => (1, -1),
            Self::West => (-1, 0),
            Self::East => (1, 0),
            Self::NorthWest => (-1, 1),
            Self::North => (0, 1),
            Self::NorthEast => (1, 1),
        }
    }
}

pub fn running_direction(walk_dir: Direction, run_dir: Direction) -> Option<u8> {
    let (wx, wy) = walk_dir.delta();
    let (rx, ry) = run_dir.delta();
    let (dx, dy) = (wx + rx, wy + ry);
    match (dx, dy) {
        (-2, -2) => Some(0),
        (-1, -2) => Some(1),
        (0, -2) => Some(2),
        (1, -2) => Some(3),
        (2, -2) => Some(4),
        (-2, -1) => Some(5),
        (2, -1) => Some(6),
        (-2, 0) => Some(7),
        (2, 0) => Some(8),
        (-2, 1) => Some(9),
        (2, 1) => Some(10),
        (-2, 2) => Some(11),
        (-1, 2) => Some(12),
        (0, 2) => Some(13),
        (1, 2) => Some(14),
        (2, 2) => Some(15),
        _ => None,
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
