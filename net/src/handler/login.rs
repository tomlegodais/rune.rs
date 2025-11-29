use crate::codec::LoginCodec;
use crate::error::SessionError;
use crate::message::{LoginInbound, LoginOutbound, LoginResponse};
use crate::service::LoginService;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct LoginHandler;

impl LoginHandler {
    pub async fn run(
        stream: TcpStream,
        hash: u8,
        service: Arc<dyn LoginService>,
    ) -> anyhow::Result<(), SessionError> {
        let (codec, session_key) = LoginCodec::with_random_key(hash);
        let mut framed = Framed::new(stream, codec);
        framed.send(LoginOutbound::SessionKey(session_key)).await?;

        let Some(request) = framed.next().await else {
            return Err(SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection closed during login",
            )));
        };

        let LoginInbound::Request(request) = request?;
        let outcome = service.authenticate(request, session_key).await?;
        let response = LoginResponse::from_outcome(outcome);

        framed.send(LoginOutbound::Response(response)).await?;

        Ok(())
    }
}
