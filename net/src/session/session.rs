use crate::LoginService;
use crate::error::SessionError;
use crate::handler::{HandshakeHandler, Js5Handler, LoginHandler, WorldListHandler};
use crate::service::CacheService;
use std::sync::Arc;
use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy)]
pub enum SessionPhase {
    Js5,
    WorldList { full_update: bool },
    Login { hash: u8 },
}

pub struct Session {
    stream: TcpStream,
    cache_service: Arc<CacheService>,
    login_service: Arc<dyn LoginService>,
}

impl Session {
    pub fn new(
        stream: TcpStream,
        cache_service: Arc<CacheService>,
        login_service: Arc<dyn LoginService>,
    ) -> Self {
        Self {
            stream,
            cache_service,
            login_service,
        }
    }

    pub async fn run(self) -> anyhow::Result<(), SessionError> {
        let (stream, phase) = HandshakeHandler::run(self.stream).await?;

        match phase {
            SessionPhase::Js5 => Js5Handler::run(stream, self.cache_service).await,
            SessionPhase::WorldList { full_update } => {
                WorldListHandler::run(stream, full_update).await
            }
            SessionPhase::Login { hash } => {
                LoginHandler::run(stream, hash, self.login_service).await
            }
        }
    }
}
