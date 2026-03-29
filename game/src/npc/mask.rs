use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::entity::{Anim, Mask, MaskConfig, MaskFlags, SpotAnim};

pub struct NpcMask;

impl NpcMask {
    pub const SPOT_ANIM_1: MaskFlags = MaskFlags(0x1);
    pub const HIT_2: MaskFlags = MaskFlags(0x2);
    pub const ANIMATION: MaskFlags = MaskFlags(0x4);
    pub const FACE_ENTITY: MaskFlags = MaskFlags(0x8);
    pub const TRANSFORMATION: MaskFlags = MaskFlags(0x10);
    pub const HIT_1: MaskFlags = MaskFlags(0x20);
    pub const EXTENDED: MaskFlags = MaskFlags(0x40);
    pub const FORCE_TALK: MaskFlags = MaskFlags(0x80);
    pub const SECONDARY_BAR: MaskFlags = MaskFlags(0x200);
    pub const FACE_WORLD_TILE: MaskFlags = MaskFlags(0x400);
    pub const SPOT_ANIM_2: MaskFlags = MaskFlags(0x800);
    pub const COLOR_CHANGE: MaskFlags = MaskFlags(0x2000);
}

pub static NPC_MASKS: MaskConfig = MaskConfig {
    order: &[
        NpcMask::TRANSFORMATION,
        NpcMask::HIT_2,
        NpcMask::FACE_WORLD_TILE,
        NpcMask::SPOT_ANIM_2,
        NpcMask::ANIMATION,
        NpcMask::COLOR_CHANGE,
        NpcMask::SPOT_ANIM_1,
        NpcMask::SECONDARY_BAR,
        NpcMask::FACE_ENTITY,
        NpcMask::FORCE_TALK,
        NpcMask::HIT_1,
    ],
    extended: &[(0x80, NpcMask::EXTENDED)],
};

pub struct AnimationMask(pub Anim);

impl Mask for AnimationMask {
    fn flag(&self) -> MaskFlags {
        NpcMask::ANIMATION
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16_le(self.0.id);
        out.put_u8_sub(self.0.speed);
    }
}

pub struct SpotAnim1Mask(pub SpotAnim);

impl Mask for SpotAnim1Mask {
    fn flag(&self) -> MaskFlags {
        NpcMask::SPOT_ANIM_1
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16_add(self.0.id);
        out.put_u32_mid_be(self.0.speed_height_hash());
        out.put_u8_sub(self.0.rotation_hash());
    }
}

pub struct SpotAnim2Mask(pub SpotAnim);

impl Mask for SpotAnim2Mask {
    fn flag(&self) -> MaskFlags {
        NpcMask::SPOT_ANIM_2
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16(self.0.id);
        out.put_u32_mid_le(self.0.speed_height_hash());
        out.put_u8_sub(self.0.rotation_hash());
    }
}

pub struct FaceEntityMask(pub u16);

impl Mask for FaceEntityMask {
    fn flag(&self) -> MaskFlags {
        NpcMask::FACE_ENTITY
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16_add(self.0);
    }
}

pub struct ForceTalkMask(pub String);

impl Mask for ForceTalkMask {
    fn flag(&self) -> MaskFlags {
        NpcMask::FORCE_TALK
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_string(&self.0);
    }
}
