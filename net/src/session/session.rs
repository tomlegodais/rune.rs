use crate::error::SessionError;
use crate::handler::{HandshakeHandler, Js5Handler, LoginHandler, WorldListHandler};
use crate::service::CacheService;
use std::sync::Arc;
use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy)]
pub enum SessionPhase {
    Js5,
    WorldList { full_update: bool },
    Login,
}

pub struct Session {
    stream: TcpStream,
    service: Arc<CacheService>,
}

impl Session {
    pub fn new(stream: TcpStream, service: Arc<CacheService>) -> Self {
        Self { stream, service }
    }

    pub async fn run(self) -> anyhow::Result<(), SessionError> {
        let (stream, phase) = HandshakeHandler::run(self.stream).await?;

        match phase {
            SessionPhase::Js5 => Js5Handler::run(stream, self.service).await,
            SessionPhase::WorldList { full_update } => {
                WorldListHandler::run(stream, full_update).await
            }
            SessionPhase::Login => LoginHandler::run(stream).await,
        }
    }
}
