use net::{Encodable, Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};
use util::{BitsMut, BytesMutExt};

pub struct GameScene {
    pub init: bool,
    pub position_bits: u32,
    pub player_id: u16,
    pub size: u8,
    pub center_chunk_x: i32,
    pub center_chunk_y: i32,
    pub region_count: usize,
}

impl Encodable for GameScene {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();

        if self.init {
            let mut bit_pos = buf.bits_start();
            buf.put_bits(&mut bit_pos, 30, self.position_bits);

            for _player_index in 1..2048 {
                if _player_index == self.player_id {
                    continue;
                }
                buf.put_bits(&mut bit_pos, 18, 0);
            }

            buf.bits_end(bit_pos);
        }

        buf.put_u8_sub(self.size);
        buf.put_u16_add(self.center_chunk_x as u16);
        buf.put_u16_le_add(self.center_chunk_y as u16);
        buf.put_u8_add(0);

        for _ in 0..self.region_count {
            for _ in 0..4 {
                buf.put_u32(0);
            }
        }

        Frame {
            opcode: 13,
            prefix: Prefix::Short,
            payload: buf.freeze(),
        }
    }
}