use crate::codec::{WorldListCodec, WorldListInbound, WorldListOutbound};
use crate::error::SessionError;
use crate::response::WorldListEncoder;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct WorldListHandler;

impl WorldListHandler {
    pub async fn run(stream: TcpStream) -> anyhow::Result<(), SessionError> {
        let codec = WorldListCodec::default();
        let mut framed = Framed::new(stream, codec);

        let Some(frame) = framed.next().await else {
            return Err(SessionError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "connection closed before world list request",
            )));
        };

        let msg = frame?;

        if let WorldListInbound { full_update } = msg {
            let response = WorldListEncoder::encode(full_update, "127.0.0.1", 100);
            let outbound = WorldListOutbound { bytes: response };
            framed.send(outbound).await?;
        }

        Ok(())
    }
}
