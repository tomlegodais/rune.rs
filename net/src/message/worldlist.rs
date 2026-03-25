use bitflags::bitflags;
use num_enum::IntoPrimitive;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct WorldFlag: u32 {
        const MEMBERS = 0x1;
        const QUICK_CHAT = 0x2;
        const PVP = 0x4;
        const LOOTSHARE = 0x8;
        const HIGHLIGHT = 0x400;
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub id: u16,
    pub location: u8,
    pub flags: WorldFlag,
    pub activity: String,
    pub hostname: String,
    pub player_count: u16,
}

#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u16)]
pub enum CountryFlag {
    Australia = 0x10,
    Belgium = 0x16,
    Brazil = 0x1F,
    Canada = 0x26,
    Denmark = 0x3A,
    Finland = 0x45,
    Ireland = 0x65,
    Uk = 0x4D,
    Mexico = 0x98,
    Netherlands = 0xA1,
    Norway = 0xA2,
    Sweden = 0xBF,
    Usa = 0xE1,
}

#[derive(Debug, Clone)]
pub struct Country {
    pub flag: CountryFlag,
    pub name: String,
}

#[derive(Debug)]
pub struct WorldListOutbound {
    pub full_update: bool,
    pub countries: Vec<Country>,
    pub worlds: Vec<World>,
    pub session_id: u32,
}
