#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HandshakeOpcode {
    Js5 = 15,
    Login = 14,
    WorldList = 23,
}

impl HandshakeOpcode {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            15 => Some(Self::Js5),
            14 => Some(Self::Login),
            23 => Some(Self::WorldList),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HandshakeResponse {
    Success = 0,
    OutOfDate = 6,
}

impl HandshakeResponse {
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}
