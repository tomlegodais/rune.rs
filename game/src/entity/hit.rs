use num_enum::IntoPrimitive;

#[derive(Clone, Copy, IntoPrimitive)]
#[repr(u8)]
pub enum HitType {
    Block = 0,
    Normal = 1,
    Poison = 2,
    Disease = 3,
}

#[derive(Clone, Copy)]
pub struct Hit {
    pub damage: u16,
    pub hit_type: HitType,
    pub hp_ratio: u8,
}

impl Hit {
    pub fn new(damage: u16, hit_type: HitType) -> Self {
        Self {
            damage,
            hit_type,
            hp_ratio: 0,
        }
    }
}
