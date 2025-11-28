use crate::codec::{HandshakeCodec, HandshakeInbound, HandshakeOutbound, HandshakeResponse};
use crate::error::SessionError;
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

        let msg = frame?;
        let phase = match msg {
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

            HandshakeInbound::WorldList => SessionPhase::WorldList,
            HandshakeInbound::Login => SessionPhase::Login,
        };

        let stream = framed.into_inner();
        Ok((stream, phase))
    }
}
