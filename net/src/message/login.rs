use crate::GameMessage;
use num_enum::IntoPrimitive;
use tokio::sync::mpsc;
use tokio_util::bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u8)]
pub enum StatusCode {
    SessionKey = 0,
    OK = 2,
    InvalidCredentials = 3,
    GameUpdated = 6,
    BadSessionId = 10,
}

#[derive(Debug)]
pub struct LoginRequest {
    pub version: u32,
    pub display_mode: u8,
    pub crc: [u32; 31],
    pub client_key: i64,
    pub server_key: i64,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginResponse {
    pub status: StatusCode,
    pub payload: Option<Bytes>,
}

#[derive(Debug)]
pub enum LoginOutcome {
    Success(LoginSuccess),
    InvalidCredentials,
    GameUpdated,
    BadSessionId,
}

#[derive(Debug)]
pub struct LoginSuccess {
    pub rights: u8,
    pub player_index: u16,
    pub members: bool,
    pub inbox_tx: mpsc::Sender<GameMessage>,
    pub outbound_rx: mpsc::Receiver<GameMessage>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum LoginState {
    Header,
    Payload,
}

#[derive(Debug)]
pub enum LoginInbound {
    Request(LoginRequest),
}

#[derive(Debug)]
pub enum LoginOutbound {
    SessionKey(i64),
    Response(LoginResponse),
}

impl LoginResponse {
    pub fn from_outcome(outcome: &LoginOutcome) -> Self {
        match outcome {
            LoginOutcome::Success(s) => LoginResponse {
                status: StatusCode::OK,
                payload: Some(s.payload()),
            },

            other => LoginResponse {
                status: other.status_code(),
                payload: None,
            },
        }
    }
}

impl LoginOutcome {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginOutcome::InvalidCredentials => StatusCode::InvalidCredentials,
            LoginOutcome::GameUpdated => StatusCode::GameUpdated,
            LoginOutcome::BadSessionId => StatusCode::BadSessionId,
            LoginOutcome::Success(_) => StatusCode::OK,
        }
    }
}

impl LoginSuccess {
    fn payload(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(10);
        buf.put_u8(self.rights);
        buf.put_u8(0);
        buf.put_u8(0);
        buf.put_u8(0);
        buf.put_u8(0);
        buf.put_u8(0);
        buf.put_u16(self.player_index);
        buf.put_u8(1);
        buf.put_u8(if self.members { 1 } else { 0 });

        buf.freeze()
    }
}

impl LoginOutbound {
    pub fn status_code(&self) -> StatusCode {
        match self {
            LoginOutbound::SessionKey(_) => StatusCode::SessionKey,
            LoginOutbound::Response(r) => r.status,
        }
    }
}
