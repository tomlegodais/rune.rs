use crate::error::SessionError;
use crate::message::WorldListOutbound;
use tokio_util::bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

#[derive(Debug, Default)]
pub struct WorldListCodec;

impl Encoder<WorldListOutbound> for WorldListCodec {
    type Error = SessionError;

    fn encode(&mut self, item: WorldListOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut buffer = Vec::with_capacity(128);

        buffer.push(1);
        buffer.push(if item.full_update { 1 } else { 0 });

        if item.full_update {
            write_smart(&mut buffer, 1); // country count
            write_smart(&mut buffer, 0); // country id
            write_jag_string(&mut buffer, "World 1"); // world name

            write_smart(&mut buffer, 0); // min world id
            write_smart(&mut buffer, 2); // max world id + 1
            write_smart(&mut buffer, 1); // world count

            write_smart(&mut buffer, 1); // world id
            buffer.push(0); // location (country index)
            buffer.extend_from_slice(&(0x1u32 | 0x8).to_be_bytes()); // flags: members | lootshare
            write_jag_string(&mut buffer, ""); // activity
            write_jag_string(&mut buffer, &item.host); // hostname
            buffer.extend_from_slice(&0x94DA4A87u32.to_be_bytes()); // session id
        }

        write_smart(&mut buffer, 1); // world count
        buffer.extend_from_slice(&item.player_count.to_be_bytes()); // player count

        dst.put_u8(0);
        dst.put_u16(buffer.len() as u16);
        dst.extend_from_slice(&buffer);

        Ok(())
    }
}

fn write_smart(buffer: &mut Vec<u8>, value: u16) {
    if value < 128 {
        buffer.push(value as u8);
    } else {
        buffer.push(((value >> 8) as u8) | 0x80);
        buffer.push(value as u8);
    }
}

fn write_jag_string(buffer: &mut Vec<u8>, s: &str) {
    buffer.push(0);
    buffer.extend_from_slice(s.as_bytes());
    buffer.push(0);
}
