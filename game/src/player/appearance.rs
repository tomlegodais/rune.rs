use crate::player::mask::{MaskEncoder, MaskFlags};
use tokio_util::bytes::{BufMut, BytesMut};

const DEFAULT_LOOK: [u16; 7] = [8, 14, 18, 26, 34, 38, 42];
const DEFAULT_COLORS: [u8; 5] = [5, 15, 15, 0, 0];
const DEFAULT_RENDER_EMOTE: u16 = 1426;

#[derive(Clone)]
pub struct Appearance {
    pub male: bool,
    pub look: [u16; 7],
    pub colors: [u8; 5],
    pub render_emote: u16,
    pub display_name: String,
    pub combat_level: u8,
}

impl Appearance {
    pub fn new(display_name: &str, combat_level: u8) -> Self {
        Self {
            male: true,
            look: DEFAULT_LOOK,
            colors: DEFAULT_COLORS,
            render_emote: DEFAULT_RENDER_EMOTE,
            display_name: display_name.to_string(),
            combat_level,
        }
    }
}

pub struct AppearanceEncoder<'a> {
    appearance: &'a Appearance,
}

impl<'a> AppearanceEncoder<'a> {
    pub fn new(appearance: &'a Appearance) -> Self {
        Self { appearance }
    }
}

impl<'a> MaskEncoder for AppearanceEncoder<'a> {
    fn flag() -> MaskFlags {
        MaskFlags::APPEARANCE
    }

    fn encode_mask(&self, out: &mut BytesMut) {
        let mut buf = BytesMut::new();
        self.encode_appearance(&mut buf);

        out.put_u8(buf.len() as u8);
        out.extend_from_slice(&buf);
    }
}

impl<'a> AppearanceEncoder<'a> {
    fn encode_appearance(&self, buf: &mut BytesMut) {
        let app = self.appearance;

        buf.put_u8(if app.male { 0 } else { 1 });
        buf.put_u8(0); // title
        buf.put_u8(0xFF); // skull icon
        buf.put_u8(0xFF); // prayer icon

        // Slots 0-3: hat, cape, amulet, weapon (empty)
        for _ in 0..4 {
            buf.put_u8(0);
        }
        buf.put_u16(0x100 | app.look[2]); // chest
        buf.put_u8(0); // shield
        buf.put_u16(0x100 | app.look[3]); // arms
        buf.put_u16(0x100 | app.look[5]); // legs
        buf.put_u16(0x100 | app.look[0]); // hair
        buf.put_u16(0x100 | app.look[4]); // hands
        buf.put_u16(0x100 | app.look[6]); // feet

        if app.male {
            buf.put_u16(0x100 | app.look[1]); // beard
        } else {
            buf.put_u8(0);
        }

        for &color in &app.colors {
            buf.put_u8(color);
        }

        buf.put_u16(app.render_emote);
        buf.put_slice(app.display_name.as_bytes());
        buf.put_u8(0);
        buf.put_u8(app.combat_level);
        buf.put_u8(0);
        buf.put_u8(0xFF);
        buf.put_u8(0);
    }
}
