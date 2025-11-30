use crate::player::Scene;
use crate::world::Position;
use net::{GameMessage, MessageType, ServerMessage};
use tokio_util::bytes::{BufMut, BytesMut};
use util::{BitsMut, BytesMutExt};

pub struct GameScene<'a> {
    pub init: bool,
    pub position: Position,
    pub scene: &'a Scene,
    pub player_id: u16,
}

impl ServerMessage for GameScene<'_> {
    fn into_game_message(self) -> GameMessage {
        let mut buf = BytesMut::new();

        if self.init {
            let mut bit_pos = buf.bits_start();
            buf.put_bits(&mut bit_pos, 30, self.position.bits());

            for _player_index in 1..2048 {
                if _player_index == self.player_id {
                    continue;
                }
                buf.put_bits(&mut bit_pos, 18, 0);
            }

            buf.bits_end(bit_pos);
        }

        buf.put_u8_sub(self.scene.size);
        buf.put_u16_add(self.scene.center_chunk_x as u16);
        buf.put_u16_le_add(self.scene.center_chunk_y as u16);
        buf.put_u8_add(0);

        for _region_id in &self.scene.region_ids {
            for _ in 0..4 {
                buf.put_u32(0);
            }
        }

        GameMessage {
            opcode: 13,
            ty: MessageType::Short,
            payload: buf.freeze(),
        }
    }
}
