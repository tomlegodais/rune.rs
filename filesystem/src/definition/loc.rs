use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

#[derive(Debug, Clone)]
pub struct LocDefinition {
    pub id: u32,
    pub name: String,
    pub size_x: u8,
    pub size_y: u8,
    pub block_walk: bool,
    pub block_range: bool,
    pub solid: u8,
    pub access_block_flag: u8,
    pub obstruct_ground: bool,
    pub unclipped: bool,
    pub interact_type: u8,
}

impl Default for LocDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            size_x: 1,
            size_y: 1,
            block_walk: true,
            block_range: true,
            solid: 2,
            access_block_flag: 0,
            obstruct_ground: false,
            unclipped: false,
            interact_type: 0,
        }
    }
}

impl LocDefinition {
    pub fn decode(id: u32, data: &[u8]) -> Self {
        let mut def = Self {
            id,
            ..Default::default()
        };

        let mut buf = Bytes::copy_from_slice(data);
        loop {
            if !buf.has_remaining() {
                break;
            }
            let opcode = buf.get_u8();
            if opcode == 0 {
                break;
            }
            def.decode_opcode(&mut buf, opcode);
        }

        def
    }

    fn decode_opcode(&mut self, buf: &mut Bytes, opcode: u8) {
        match opcode {
            1 | 5 => {
                let count = buf.get_u8() as usize;
                for _ in 0..count {
                    buf.advance(1);
                    let inner = buf.get_u8() as usize;
                    buf.advance(inner * 2);
                }
                if opcode == 5 {
                    let extra = buf.get_u8() as usize;
                    for _ in 0..extra {
                        buf.advance(1);
                        let inner = buf.get_u8() as usize;
                        buf.advance(inner * 2);
                    }
                }
            }
            2 => {
                self.name = buf.get_string();
            }
            14 => {
                self.size_x = buf.get_u8();
            }
            15 => {
                self.size_y = buf.get_u8();
            }
            17 => {
                self.block_walk = false;
                self.solid = 0;
            }
            18 => {
                self.block_range = false;
            }
            19 => {
                self.interact_type = buf.get_u8();
            }
            21 | 22 | 23 => {}
            24 => {
                buf.advance(2);
            }
            27 => {
                self.solid = 1;
            }
            28 => {
                buf.advance(1);
            }
            29 => {
                buf.advance(1);
            }
            30..=34 => {
                buf.get_string();
            }
            39 => {
                buf.advance(1);
            }
            40 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 4);
            }
            41 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 4);
            }
            42 => {
                let count = buf.get_u8() as usize;
                buf.advance(count);
            }
            62 | 64 => {}
            65 | 66 | 67 => {
                buf.advance(2);
            }
            69 => {
                self.access_block_flag = buf.get_u8();
            }
            70 | 71 | 72 => {
                buf.advance(2);
            }
            73 => {
                self.obstruct_ground = true;
            }
            74 => {
                self.unclipped = true;
            }
            75 => {
                buf.advance(1);
            }
            77 | 92 => {
                buf.advance(4);
                if opcode == 92 {
                    buf.advance(2);
                }
                let count = buf.get_u8();
                buf.advance((count as usize + 1) * 2);
            }
            78 => {
                buf.advance(3);
            }
            79 => {
                buf.advance(5);
                let count = buf.get_u8() as usize;
                buf.advance(count * 2);
            }
            81 => {
                buf.advance(1);
            }
            82 | 88 | 89 | 90 | 91 => {}
            93 => {
                buf.advance(2);
            }
            94 | 95 | 96 | 97 | 98 => {}
            99 | 100 => {
                buf.advance(3);
            }
            101 => {
                buf.advance(1);
            }
            102 => {
                buf.advance(2);
            }
            103 => {
                self.solid = 0;
            }
            104 => {
                buf.advance(1);
            }
            105 => {}
            106 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 3);
            }
            107 => {
                buf.advance(2);
            }
            150..=154 => {
                buf.get_string();
            }
            160 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 2);
            }
            162 => {
                buf.advance(4);
            }
            163 => {
                buf.advance(4);
            }
            249 => {
                let count = buf.get_u8() as usize;
                for _ in 0..count {
                    let is_string = buf.get_u8() == 1;
                    buf.advance(3);
                    if is_string {
                        buf.get_string();
                    } else {
                        buf.advance(4);
                    }
                }
            }
            _ => {}
        }
    }
}
