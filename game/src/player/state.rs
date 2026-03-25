use crate::entity::MaskBlock;
use crate::entity::MoveStep;
use crate::player::mask;
use crate::world::Teleport;

pub const MAX_PLAYERS: usize = 2048;

#[derive(Clone)]
pub struct PlayerState {
    pub local: bool,
    pub activity: u8,
    pub region_hash: u32,
    pub teleport: Option<Teleport>,
    pub move_step: MoveStep,
    pub masks: MaskBlock,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            local: false,
            activity: 0,
            region_hash: 0,
            teleport: None,
            move_step: MoveStep::default(),
            masks: MaskBlock::new(&mask::PLAYER_MASKS),
        }
    }
}
