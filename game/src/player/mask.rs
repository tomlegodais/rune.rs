use crate::player::Appearance;
use crate::provider;
use num_enum::IntoPrimitive;
use persistence::Rights;
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct MaskFlags(u32);

impl MaskFlags {
    pub const MOVE_TYPE: Self = Self(0x1);
    pub const FACE_ENTITY: Self = Self(0x2);
    pub const EXTENDED: Self = Self(0x4);
    pub const ANIMATION: Self = Self(0x8);
    pub const HIT_1: Self = Self(0x10);
    pub const APPEARANCE: Self = Self(0x20);
    pub const FACE_DIRECTION: Self = Self(0x40);
    pub const CHAT: Self = Self(0x80);
    pub const HIT_2: Self = Self(0x100);
    pub const TEMP_MOVE_TYPE: Self = Self(0x200);
    pub const FORCE_TALK: Self = Self(0x400);
    pub const GRAPHICS_1: Self = Self(0x1000);
    pub const EXTENDED_2: Self = Self(0x2000);
    pub const GRAPHICS_2: Self = Self(0x40000);

    pub const EMPTY: Self = Self(0);

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    fn wire_value(self) -> u32 {
        let mut v = self.0;
        if v > 128 {
            v |= Self::EXTENDED.0;
        }
        if v > 32768 {
            v |= Self::EXTENDED_2.0;
        }
        v
    }
}

impl std::ops::BitOr for MaskFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

pub trait Mask {
    fn flag(&self) -> MaskFlags;
    fn encode(&self, out: &mut BytesMut);
}

impl<T: Mask + ?Sized> Mask for &T {
    fn flag(&self) -> MaskFlags {
        (*self).flag()
    }
    fn encode(&self, out: &mut BytesMut) {
        (*self).encode(out)
    }
}

#[derive(Clone)]
struct EncodedMask {
    flag: MaskFlags,
    data: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct MaskBlock {
    flags: MaskFlags,
    masks: Vec<EncodedMask>,
}

const MASK_ORDER: &[MaskFlags] = &[
    MaskFlags::FACE_DIRECTION,
    MaskFlags::FORCE_TALK,
    MaskFlags::GRAPHICS_2,
    MaskFlags::MOVE_TYPE,
    MaskFlags::FACE_ENTITY,
    MaskFlags::CHAT,
    MaskFlags::GRAPHICS_1,
    MaskFlags::ANIMATION,
    MaskFlags::TEMP_MOVE_TYPE,
    MaskFlags::HIT_1,
    MaskFlags::APPEARANCE,
    MaskFlags::HIT_2,
];

impl MaskBlock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&self, out: &mut BytesMut) {
        let wire = self.flags.wire_value();

        if wire > 128 {
            out.put_u8((wire & 0xFF) as u8);
            out.put_u8((wire >> 8) as u8);
        } else {
            out.put_u8(wire as u8);
        }

        for &order_flag in MASK_ORDER {
            if !self.flags.contains(order_flag) {
                continue;
            }
            if let Some(mask) = self.masks.iter().find(|m| m.flag == order_flag) {
                out.put_slice(&mask.data);
            }
        }
    }

    pub fn add(&mut self, mask: impl Mask) {
        let flag = mask.flag();
        let mut buf = BytesMut::new();
        mask.encode(&mut buf);

        self.flags = self.flags | flag;
        self.masks.push(EncodedMask {
            flag,
            data: buf.to_vec(),
        });
    }

    pub fn extend(&mut self, masks: &[&dyn Mask]) {
        masks.iter().for_each(|m| self.add(*m));
    }

    pub fn clear(&mut self) {
        self.flags = MaskFlags::EMPTY;
        self.masks.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }
}

pub struct MoveTypeMask(pub bool);

impl Mask for MoveTypeMask {
    fn flag(&self) -> MaskFlags {
        MaskFlags::MOVE_TYPE
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
        MaskFlags::TEMP_MOVE_TYPE
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
        MaskFlags::APPEARANCE
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
        MaskFlags::CHAT
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
