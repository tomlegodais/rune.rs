use crate::world::Teleport;

pub const MAX_PLAYERS: usize = 2048;

#[derive(Copy, Clone)]
pub struct PlayerState {
    pub local: bool,
    pub activity: u8,
    pub region_hash: u32,
    pub teleport: Option<Teleport>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            local: false,
            activity: 0,
            region_hash: 0,
            teleport: None,
        }
    }
}