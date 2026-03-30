use filesystem::definition::{EquipmentFlag, ParamMap};
use num_enum::IntoPrimitive;
use persistence::Rights;
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

use crate::{
    entity::{Anim, Mask, MaskConfig, MaskFlags, SpotAnim},
    player::{DEFAULT_RENDER_EMOTE, EquipSlots, equipment::EquipmentSlot},
    provider,
    world::Direction,
};

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
    pub const SPOT_ANIM_1: MaskFlags = MaskFlags(0x1000);
    pub const EXTENDED_2: MaskFlags = MaskFlags(0x2000);
    pub const SPOT_ANIM_2: MaskFlags = MaskFlags(0x40000);
}

pub static PLAYER_MASKS: MaskConfig = MaskConfig {
    order: &[
        PlayerMask::FACE_DIRECTION,
        PlayerMask::FORCE_TALK,
        PlayerMask::SPOT_ANIM_2,
        PlayerMask::MOVE_TYPE,
        PlayerMask::FACE_ENTITY,
        PlayerMask::CHAT,
        PlayerMask::SPOT_ANIM_1,
        PlayerMask::ANIMATION,
        PlayerMask::TEMP_MOVE_TYPE,
        PlayerMask::HIT_1,
        PlayerMask::APPEARANCE,
        PlayerMask::HIT_2,
    ],
    extended: &[(0x80, PlayerMask::EXTENDED), (0x8000, PlayerMask::EXTENDED_2)],
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

pub struct AppearanceMask {
    pub male: bool,
    pub look: [u16; 7],
    pub colors: [u8; 5],
    pub display_name: String,
    pub combat_level: u8,
    pub equipment: EquipSlots,
}

impl Mask for AppearanceMask {
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

impl AppearanceMask {
    fn encode_appearance(&self, buf: &mut BytesMut) {
        buf.put_u8(if self.male { 0 } else { 1 });
        buf.put_u8(0);
        buf.put_u8(0xFF);
        buf.put_u8(0xFF);

        let head_flag = self.flag_for(EquipmentSlot::Head);
        let hides_arms = self.equipment[EquipmentSlot::Body].is_some()
            && !matches!(self.flag_for(EquipmentSlot::Body), Some(EquipmentFlag::Sleeveless));

        self.encode_slot(buf, EquipmentSlot::Head, None);
        self.encode_slot(buf, EquipmentSlot::Cape, None);
        self.encode_slot(buf, EquipmentSlot::Amulet, None);
        self.encode_slot(buf, EquipmentSlot::Weapon, None);
        self.encode_slot(buf, EquipmentSlot::Body, Some(self.look[2]));
        self.encode_slot(buf, EquipmentSlot::Shield, None);

        if hides_arms {
            buf.put_u8(0);
        } else {
            buf.put_u16(0x100 | self.look[3]);
        }

        self.encode_slot(buf, EquipmentSlot::Legs, Some(self.look[5]));

        match head_flag {
            Some(EquipmentFlag::Hair | EquipmentFlag::FullFace) => buf.put_u8(0),
            Some(EquipmentFlag::HairMid) => buf.put_u16(0x100 | provider::get_hair_mid(self.look[0], self.male)),
            Some(EquipmentFlag::HairLow) => buf.put_u16(0x100 | provider::get_hair_low(self.look[0], self.male)),
            _ => buf.put_u16(0x100 | self.look[0]),
        }

        self.encode_slot(buf, EquipmentSlot::Gloves, Some(self.look[4]));
        self.encode_slot(buf, EquipmentSlot::Boots, Some(self.look[6]));

        let hides_beard = matches!(head_flag, Some(EquipmentFlag::FullFace | EquipmentFlag::Mask))
            || (!self.male && self.equipment[EquipmentSlot::Body].is_some());
        if hides_beard {
            buf.put_u8(0);
        } else {
            buf.put_u16(0x100 | self.look[1]);
        }

        for &color in &self.colors {
            buf.put_u8(color);
        }

        buf.put_u16(self.render_emote());
        buf.put_string(&self.display_name);
        buf.put_u8(self.combat_level);
        buf.put_u8(0);
        buf.put_u8(0xFF);
        buf.put_u8(0);
    }

    fn flag_for(&self, slot: EquipmentSlot) -> Option<EquipmentFlag> {
        self.equipment[slot]
            .and_then(|item| provider::get_item_definition(item.id as u32))
            .map(|def| def.equipment_flag)
            .filter(|f| *f != EquipmentFlag::None)
    }

    fn encode_slot(&self, buf: &mut BytesMut, slot: EquipmentSlot, kit_fallback: Option<u16>) {
        match self.equipment[slot] {
            Some(item) => buf.put_u16(0x4000 + item.id),
            None => match kit_fallback {
                Some(kit) => buf.put_u16(0x100 | kit),
                None => buf.put_u8(0),
            },
        }
    }

    fn render_emote(&self) -> u16 {
        self.equipment[EquipmentSlot::Weapon]
            .and_then(|item| provider::get_item_definition(item.id as u32))
            .and_then(|def| def.params.int_param(644))
            .map(|v| v as u16)
            .unwrap_or(DEFAULT_RENDER_EMOTE)
    }
}

pub struct AnimationMask(pub Anim);

impl Mask for AnimationMask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::ANIMATION
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16(self.0.id);
        out.put_u8(self.0.speed);
    }
}

pub struct SpotAnim1Mask(pub SpotAnim);

impl Mask for SpotAnim1Mask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::SPOT_ANIM_1
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16_le(self.0.id);
        out.put_u32(self.0.speed_height_hash());
        out.put_u8_sub(self.0.rotation_hash());
    }
}

pub struct SpotAnim2Mask(pub SpotAnim);

impl Mask for SpotAnim2Mask {
    fn flag(&self) -> MaskFlags {
        PlayerMask::SPOT_ANIM_2
    }

    fn encode(&self, out: &mut BytesMut) {
        out.put_u16_le(self.0.id);
        out.put_u32_mid_be(self.0.speed_height_hash());
        out.put_u8_neg(self.0.rotation_hash());
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
