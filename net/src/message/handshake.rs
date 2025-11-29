use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum HandshakeType {
    Js5 = 15,
    WorldList = 23,
    Login = 14,
}

impl HandshakeType {
    pub fn len(&self) -> usize {
        match self {
            HandshakeType::Js5 => 4,
            HandshakeType::WorldList => 1,
            HandshakeType::Login => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u8)]
pub enum HandshakeResponse {
    Success = 0,
    OutOfDate = 6,
}

#[derive(Debug)]
pub enum HandshakeInbound {
    Js5 { client_version: u32 },
    WorldList { full_update: bool },
    Login { hash: u8 },
}

pub enum HandshakeOutbound {
    Response(HandshakeResponse),
}
