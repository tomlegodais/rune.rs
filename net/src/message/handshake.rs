use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum HandshakeType {
    Js5 = 15,
    WorldList = 23,
    Login = 14,
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
    Login,
}

pub enum HandshakeOutbound {
    Response(HandshakeResponse),
}
