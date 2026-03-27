use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

#[derive(Debug, Clone)]
pub struct NpcDefinition {
    pub id: u32,
    pub name: String,
    pub size: u8,
    pub options: [Option<String>; 5],
    pub combat_level: i16,
    pub visible_on_map: bool,
}

impl Default for NpcDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            size: 1,
            options: [None, None, None, None, None],
            combat_level: -1,
            visible_on_map: true,
        }
    }
}

impl NpcDefinition {
    pub fn decode(id: u32, data: &[u8]) -> anyhow::Result<Self> {
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
            def.decode_opcode(&mut buf, opcode)
                .map_err(|e| anyhow::anyhow!("npc {} opcode {}: {}", id, opcode, e))?;
        }

        Ok(def)
    }

    fn decode_opcode(&mut self, buf: &mut Bytes, opcode: u8) -> anyhow::Result<()> {
        match opcode {
            1 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 2);
            }
            2 => self.name = buf.get_string(),
            12 => self.size = buf.get_u8(),
            30..=34 => {
                let opt = buf.get_string();
                self.options[(opcode - 30) as usize] = if opt.eq_ignore_ascii_case("Hidden") {
                    None
                } else {
                    Some(opt)
                };
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
            44 | 45 => {
                buf.advance(2);
            }
            60 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 2);
            }
            93 => self.visible_on_map = false,
            95 => self.combat_level = buf.get_u16() as i16,
            97 => buf.advance(2),
            98 => buf.advance(2),
            99 => {}
            100 => buf.advance(1),
            101 => buf.advance(1),
            102 => buf.advance(2),
            103 => buf.advance(2),
            106 | 118 => {
                buf.advance(2 + 2);
                if opcode == 118 {
                    buf.advance(2);
                }
                let count = buf.get_u8() as usize;
                buf.advance((count + 1) * 2);
            }
            107 | 109 | 111 => {}
            113 => buf.advance(4),
            114 => buf.advance(2),
            115 => buf.advance(2),
            119 => buf.advance(1),
            121 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 4);
            }
            122 => buf.advance(2),
            123 => buf.advance(2),
            125 => buf.advance(1),
            127 => buf.advance(2),
            128 => buf.advance(1),
            134 => buf.advance(9),
            135 => buf.advance(3),
            136 => buf.advance(3),
            137 => buf.advance(2),
            138 | 139 => buf.advance(2),
            140 => buf.advance(1),
            141..=143 => {}
            150..=154 => {
                let opt = buf.get_string();
                self.options[(opcode - 150) as usize] = if opt.eq_ignore_ascii_case("Hidden") {
                    None
                } else {
                    Some(opt)
                };
            }
            155 => buf.advance(4),
            158 | 159 | 162 => {}
            160 => {
                let count = buf.get_u8() as usize;
                buf.advance(count * 2);
            }
            163 => buf.advance(1),
            164 => buf.advance(4),
            165 | 168 => buf.advance(1),
            170..=175 => buf.advance(2),
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
        Ok(())
    }
}
