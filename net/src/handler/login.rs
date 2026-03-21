use crate::LoginOutcome;
use crate::codec::LoginCodec;
use crate::crypto::NoopCipher;
use crate::error::SessionError;
use crate::handler::GameHandler;
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
        let response = LoginResponse::from_outcome(&outcome);
        framed.send(LoginOutbound::Response(response)).await?;

        match outcome {
            LoginOutcome::Success(s) => {
                let player_index = s.player_index;
                let parts = framed.into_parts();
                let stream = parts.io;
                let in_cipher = NoopCipher;
                let out_cipher = NoopCipher;

                let result =
                    GameHandler::run(stream, in_cipher, out_cipher, s.inbox_tx, s.outbound_rx)
                        .await;

                service.logout(player_index).await;

                result
            }
            _ => Ok(()),
        }
    }
}
