use crate::error::SessionError;
use crate::message::WorldListOutbound;
use tokio_util::bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;
use util::BytesMutExt;

#[derive(Debug, Default)]
pub struct WorldListCodec;

impl Encoder<WorldListOutbound> for WorldListCodec {
    type Error = SessionError;

    fn encode(&mut self, item: WorldListOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buffer = BytesMut::new();
        buffer.put_u8(1);
        buffer.put_u8(if item.full_update { 1 } else { 0 });

        let min_world_id = item.worlds.iter().map(|w| w.id).min().unwrap_or(0);
        let max_world_id = item.worlds.iter().map(|w| w.id).max().unwrap_or(0);

        if item.full_update {
            buffer.put_smart(item.countries.len() as u16);
            for country in &item.countries {
                buffer.put_smart(country.flag as u16);
                buffer.put_versioned_string(0, &country.name);
            }

            buffer.put_smart(min_world_id);
            buffer.put_smart(max_world_id);
            buffer.put_smart(item.worlds.len() as u16);

            for world in &item.worlds {
                buffer.put_smart(world.id - min_world_id);
                buffer.put_u8(world.location);
                buffer.put_u32(world.flags.bits());
                buffer.put_versioned_string(0, &world.activity);
                buffer.put_versioned_string(0, &world.hostname);
            }

            buffer.put_u32(item.session_id);
        }

        for world in &item.worlds {
            buffer.put_smart(world.id - min_world_id);
            buffer.put_u16(world.player_count);
        }

        dst.put_u8(0);
        dst.put_u16(buffer.len() as u16);
        dst.put(buffer.freeze());

        Ok(())
    }
}
