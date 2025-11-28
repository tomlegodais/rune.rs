use crate::codec::HandshakeCodec;
use crate::error::SessionError;
use crate::message::{HandshakeInbound, HandshakeOutbound, HandshakeResponse};
use crate::session::SessionPhase;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct HandshakeHandler;

impl HandshakeHandler {
    pub async fn run(stream: TcpStream) -> anyhow::Result<(TcpStream, SessionPhase), SessionError> {
        let codec = HandshakeCodec::default();
        let mut framed = Framed::new(stream, codec);

        let Some(frame) = framed.next().await else {
            return Err(SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection closed during handshake",
            )));
        };

        let phase = match frame? {
            HandshakeInbound::Js5 { client_version } => {
                let response = match client_version == 592 {
                    true => HandshakeResponse::Success,
                    false => HandshakeResponse::OutOfDate,
                };

                framed.send(HandshakeOutbound::Response(response)).await?;

                if response != HandshakeResponse::Success {
                    return Err(SessionError::VersionMismatch);
                }

                SessionPhase::Js5
            }

            HandshakeInbound::WorldList { full_update } => SessionPhase::WorldList { full_update },
            HandshakeInbound::Login => SessionPhase::Login,
        };

        let parts = framed.into_parts();
        Ok((parts.io, phase))
    }
}
