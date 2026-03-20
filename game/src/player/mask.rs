use crate::player::AppearanceEncoder;
use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Clone, Copy)]
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

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// Returns the wire representation with extension bits set as needed.
    pub fn wire_value(self) -> u32 {
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

pub trait MaskEncoder {
    fn flag() -> MaskFlags;

    fn encode_mask(&self, out: &mut BytesMut);
}

pub struct MoveTypeMask;

impl MaskEncoder for MoveTypeMask {
    fn flag() -> MaskFlags {
        MaskFlags::MOVE_TYPE
    }

    fn encode_mask(&self, out: &mut BytesMut) {
        out.put_u8(0u8.wrapping_sub(1));
    }
}

pub struct MaskBlock<'a> {
    pub flags: &'a MaskFlags,
    pub move_type: &'a MoveTypeMask,
    pub appearance: &'a AppearanceEncoder<'a>,
}
