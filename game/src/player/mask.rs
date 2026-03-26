use crate::entity::{Mask, MaskConfig, MaskFlags};
use crate::player::Appearance;
use crate::provider;
use crate::world::Direction;
use num_enum::IntoPrimitive;
use persistence::Rights;
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

pub struct PlayerMask;

impl PlayerMask {
    pub const MOVE_TYPE: MaskFlags = MaskFlags(0x1);
    pub const FACE_ENTITY: MaskFlags = MaskFlags(0x2);
    pub const EXTENDED: MaskFlags = MaskFlags(0x4);
    pub const ANIMATION: MaskFlags = MaskFlags(0x8);
    pub const HIT_1: MaskFlags = MaskFlags(0x10);
    pub const APPEARANCE: MaskFlags = MaskFlags(0x20);
    pub const FACE_DIRECTION: MaskFlags = MaskFlags(0x40);
    pub const CHAT: MaskFlags = MaskFlags(0x80);
    pub const HIT_2: MaskFlags = MaskFlags(0x100);
    pub const TEMP_MOVE_TYPE: MaskFlags = MaskFlags(0x200);
    pub const FORCE_TALK: MaskFlags = MaskFlags(0x400);
    pub const GRAPHICS_1: MaskFlags = MaskFlags(0x1000);
    pub const EXTENDED_2: MaskFlags = MaskFlags(0x2000);
    pub const GRAPHICS_2: MaskFlags = MaskFlags(0x40000);
}

pub static PLAYER_MASKS: MaskConfig = MaskConfig {
    order: &[
        PlayerMask::FACE_DIRECTION,
        PlayerMask::FORCE_TALK,
        PlayerMask::GRAPHICS_2,
        PlayerMask::MOVE_TYPE,
        PlayerMask::FACE_ENTITY,
        PlayerMask::CHAT,
        PlayerMask::GRAPHICS_1,
        PlayerMask::ANIMATION,
        PlayerMask::TEMP_MOVE_TYPE,
        PlayerMask::HIT_1,
        PlayerMask::APPEARANCE,
        PlayerMask::HIT_2,
    ],
    extended: &[
        (0x80, PlayerMask::EXTENDED),
        (0x8000, PlayerMask::EXTENDED_2),
    ],
};

pub struct FaceEntityMask(pub u16);

impl Mask for FaceEntityMask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::FACE_ENTITY
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16_le_add(self.0);
    }
}

pub struct FaceDirectionMask(pub Direction);

impl Mask for FaceDirectionMask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::FACE_DIRECTION
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16(self.0.to_angle());
    }
}

pub struct MoveTypeMask(pub bool);

impl Mask for MoveTypeMask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::MOVE_TYPE
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u8(0u8.wrapping_sub(if self.0 { 2 } else { 1 }));
    }
}

#[derive(Clone, Copy, IntoPrimitive)]
#[repr(u8)]
pub enum TempMoveTypeMask {
    Walk = 1,
    Run = 2,
    Teleport = 127,
}

impl Mask for TempMoveTypeMask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::TEMP_MOVE_TYPE
    }

    fn encode(&self, out: &mut BytesMut) {
        let value: u8 = (*self).into();
        out.put_u8(0u8.wrapping_sub(value));
    }
}

pub struct AppearanceMask<'a> {
    appearance: &'a Appearance,
}

impl<'a> AppearanceMask<'a> {
    pub fn new(appearance: &'a Appearance) -> Self {
        Self { appearance }
    }
}

impl Mask for AppearanceMask<'_> {
    fn flag(&self) -> MaskFlags {
        PlayerMask::APPEARANCE
    }

    fn encode(&self, out: &mut BytesMut) {
        let mut buf = BytesMut::new();
        self.encode_appearance(&mut buf);

        out.put_u8(buf.len() as u8);
        out.extend_from_slice(&buf);
    }
}

impl<'a> AppearanceMask<'a> {
    fn encode_appearance(&self, buf: &mut BytesMut) {
        let app = self.appearance;

        buf.put_u8(if app.male { 0 } else { 1 });
        buf.put_u8(0); // title
        buf.put_u8(0xFF); // skull icon
        buf.put_u8(0xFF); // prayer icon

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
        buf.put_string(&app.display_name);
        buf.put_u8(app.combat_level);
        buf.put_u8(0);
        buf.put_u8(0xFF);
        buf.put_u8(0);
    }
}

pub struct ChatMask {
    pub message: String,
    pub color: u8,
    pub effect: u8,
    pub rights: Rights,
}

impl Mask for ChatMask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::CHAT
    }

    fn encode(&self, out: &mut BytesMut) {
        let effects = ((self.color as u16) << 8) | self.effect as u16;
        out.put_u16_add(effects);
        out.put_u8_add(self.rights.into());

        let encoded = provider::encode_huffman(&self.message);
        let total = 1 + encoded.len();
        out.put_u8_add(total as u8);
        out.put_smart(self.message.len() as u16);
        out.put_slice(&encoded);
    }
}
