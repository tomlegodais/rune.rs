use crate::codec::WorldListCodec;
use crate::error::SessionError;
use crate::message::{Country, CountryFlag, World, WorldFlag, WorldListOutbound};
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
            countries: vec![Country {
                flag: CountryFlag::USA,
                name: "USA".to_string(),
            }],
            worlds: vec![World {
                id: 1,
                location: 0,
                flags: WorldFlag::MEMBERS | WorldFlag::LOOTSHARE,
                activity: "Members".to_string(),
                hostname: "127.0.0.1".to_string(),
                player_count: 0,
            }],
            session_id: 0xDEADBEEF,
        };

        framed.send(outbound).await?;

        Ok(())
    }
}
