use crate::codec::WorldListCodec;
use crate::error::SessionError;
use crate::message::WorldListOutbound;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct WorldListHandler;

impl WorldListHandler {
    pub async fn run(stream: TcpStream, full_update: bool) -> anyhow::Result<(), SessionError> {
        let codec = WorldListCodec::default();
        let mut framed = Framed::new(stream, codec);

        let outbound = WorldListOutbound {
            full_update,
            host: "127.0.0.1".to_string(),
            player_count: 100,
        };

        framed.send(outbound).await?;

        Ok(())
    }
}
