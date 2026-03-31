use tokio_util::bytes::{BufMut, BytesMut};
use util::{BitsMut, BytesMutExt};

use crate::{Encodable, Frame, Prefix};

pub struct RebuildNormal {
    pub init: bool,
    pub position_bits: u32,
    pub player_index: usize,
    pub view_distance: usize,
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub region_count: usize,
    pub region_hashes: [u32; 2048],
}

impl Encodable for RebuildNormal {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();

        if self.init {
            let mut bit_pos = buf.bits_start();
            buf.put_bits(&mut bit_pos, 30, self.position_bits);

            for player_index in 1..2048usize {
                if player_index == self.player_index {
                    continue;
                }
                buf.put_bits(&mut bit_pos, 18, self.region_hashes[player_index]);
            }

            buf.bits_end(bit_pos);
        }

        buf.put_u8_sub(self.view_distance as u8);
        buf.put_u16_add(self.chunk_x as u16);
        buf.put_u16_le_add(self.chunk_y as u16);
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
