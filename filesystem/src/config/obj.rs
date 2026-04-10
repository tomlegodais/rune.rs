use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use crate::config::ParamValue;

pub enum TransformKind {
    Noted,
    Lent,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EquipBonuses {
    pub atk_stab: i16,
    pub atk_slash: i16,
    pub atk_crush: i16,
    pub atk_magic: i16,
    pub atk_ranged: i16,
    pub def_stab: i16,
    pub def_slash: i16,
    pub def_crush: i16,
    pub def_magic: i16,
    pub def_ranged: i16,
    pub str_bonus: i16,
    pub ranged_str: i16,
    pub magic_dmg: i16,
    pub prayer: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Stab,
    Slash,
    Crush,
    Magic,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponStance {
    Accurate,
    Aggressive,
    Controlled,
    Defensive,
    Rapid,
    Longrange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpType {
    Attack,
    Strength,
    Defence,
    Ranged,
    RangedAndDefence,
    Magic,
    MagicAndDefence,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleName {
    Accurate,
    Bash,
    Blaze,
    Block,
    Chop,
    Deflect,
    Explosive,
    Fend,
    Flamer,
    Flare,
    Flick,
    Focus,
    Hack,
    Impale,
    Jab,
    Kick,
    Lash,
    Longrange,
    LongFuse,
    Lunge,
    MediumFuse,
    Pound,
    Pummel,
    Punch,
    Rapid,
    Reap,
    Scorch,
    ShortFuse,
    Slash,
    Smash,
    Spell,
    Spike,
    Stab,
    Swipe,
}

#[derive(Debug, Clone, Copy)]
pub struct CombatStyle {
    pub name: StyleName,
    pub atk_type: Option<AttackType>,
    pub stance: Option<WeaponStance>,
    pub xp_type: Option<XpType>,
}

impl CombatStyle {
    const fn new(name: StyleName, atk_type: AttackType, stance: WeaponStance, xp_type: XpType) -> Self {
        Self {
            name,
            atk_type: Some(atk_type),
            stance: Some(stance),
            xp_type: Some(xp_type),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponCategory {
    TwoHandedSword,
    Axe,
    Banner,
    Blunt,
    Bludgeon,
    Bulwark,
    Claw,
    Egg,
    Partisan,
    Pickaxe,
    Polearm,
    Polestaff,
    Scythe,
    SlashSword,
    Spear,
    Spiked,
    StabSword,
    Unarmed,
    Whip,
    Bow,
    Blaster,
    Chinchompa,
    Crossbow,
    Gun,
    Thrown,
    BladedStaff,
    PoweredStaff,
    Staff,
    Salamander,
    MultiStyle,
}

impl WeaponCategory {
    pub fn combat_styles(self) -> &'static [CombatStyle] {
        use AttackType as A;
        use StyleName as S;
        use WeaponStance as St;
        use XpType as X;
        match self {
            Self::TwoHandedSword => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Chop, A::Slash, St::Accurate, X::Attack),
                    CombatStyle::new(S::Slash, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Smash, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Slash, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Axe => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Chop, A::Slash, St::Accurate, X::Attack),
                    CombatStyle::new(S::Hack, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Smash, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Slash, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Banner => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Lunge, A::Stab, St::Accurate, X::Attack),
                    CombatStyle::new(S::Swipe, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Pound, A::Crush, St::Controlled, X::Shared),
                    CombatStyle::new(S::Block, A::Stab, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Blunt | Self::Egg => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Pound, A::Crush, St::Accurate, X::Attack),
                    CombatStyle::new(S::Pummel, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Crush, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Bludgeon => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Pound, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Pummel, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Smash, A::Crush, St::Aggressive, X::Strength),
                ];
                V
            }
            Self::Bulwark => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Pummel, A::Crush, St::Accurate, X::Attack),
                    CombatStyle {
                        name: S::Block,
                        atk_type: None,
                        stance: None,
                        xp_type: None,
                    },
                ];
                V
            }
            Self::Claw => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Chop, A::Slash, St::Accurate, X::Attack),
                    CombatStyle::new(S::Slash, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Lunge, A::Stab, St::Controlled, X::Shared),
                    CombatStyle::new(S::Block, A::Slash, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Partisan => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Stab, A::Stab, St::Accurate, X::Attack),
                    CombatStyle::new(S::Lunge, A::Stab, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Pound, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Stab, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Pickaxe => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Spike, A::Stab, St::Accurate, X::Attack),
                    CombatStyle::new(S::Impale, A::Stab, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Smash, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Stab, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Polearm => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Jab, A::Stab, St::Controlled, X::Shared),
                    CombatStyle::new(S::Swipe, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Fend, A::Stab, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Polestaff => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Bash, A::Crush, St::Accurate, X::Attack),
                    CombatStyle::new(S::Pound, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Crush, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Scythe => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Reap, A::Slash, St::Accurate, X::Attack),
                    CombatStyle::new(S::Chop, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Jab, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Slash, St::Defensive, X::Defence),
                ];
                V
            }
            Self::SlashSword => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Chop, A::Slash, St::Accurate, X::Attack),
                    CombatStyle::new(S::Slash, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Lunge, A::Stab, St::Controlled, X::Shared),
                    CombatStyle::new(S::Block, A::Slash, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Spear => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Lunge, A::Stab, St::Controlled, X::Shared),
                    CombatStyle::new(S::Swipe, A::Slash, St::Controlled, X::Shared),
                    CombatStyle::new(S::Pound, A::Crush, St::Controlled, X::Shared),
                    CombatStyle::new(S::Block, A::Stab, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Spiked => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Pound, A::Crush, St::Accurate, X::Attack),
                    CombatStyle::new(S::Pummel, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Spike, A::Stab, St::Controlled, X::Shared),
                    CombatStyle::new(S::Block, A::Crush, St::Defensive, X::Defence),
                ];
                V
            }
            Self::StabSword => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Stab, A::Stab, St::Accurate, X::Attack),
                    CombatStyle::new(S::Lunge, A::Stab, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Slash, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Stab, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Unarmed => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Punch, A::Crush, St::Accurate, X::Attack),
                    CombatStyle::new(S::Kick, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Block, A::Crush, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Whip => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Flick, A::Slash, St::Accurate, X::Attack),
                    CombatStyle::new(S::Lash, A::Slash, St::Controlled, X::Shared),
                    CombatStyle::new(S::Deflect, A::Slash, St::Defensive, X::Defence),
                ];
                V
            }
            Self::Bow | Self::Crossbow | Self::Thrown | Self::Gun | Self::MultiStyle => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Accurate, A::Ranged, St::Accurate, X::Ranged),
                    CombatStyle::new(S::Rapid, A::Ranged, St::Rapid, X::Ranged),
                    CombatStyle::new(S::Longrange, A::Ranged, St::Longrange, X::RangedAndDefence),
                ];
                V
            }
            Self::Chinchompa => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::ShortFuse, A::Ranged, St::Accurate, X::Ranged),
                    CombatStyle::new(S::MediumFuse, A::Ranged, St::Rapid, X::Ranged),
                    CombatStyle::new(S::LongFuse, A::Ranged, St::Longrange, X::RangedAndDefence),
                ];
                V
            }
            Self::Blaster => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Explosive, A::Ranged, St::Accurate, X::Ranged),
                    CombatStyle::new(S::Flamer, A::Ranged, St::Rapid, X::Ranged),
                ];
                V
            }
            Self::Staff => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Bash, A::Crush, St::Accurate, X::Attack),
                    CombatStyle::new(S::Pound, A::Crush, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Focus, A::Crush, St::Defensive, X::Defence),
                    CombatStyle::new(S::Spell, A::Magic, St::Accurate, X::Magic),
                    CombatStyle::new(S::Spell, A::Magic, St::Defensive, X::MagicAndDefence),
                ];
                V
            }
            Self::BladedStaff => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Jab, A::Stab, St::Accurate, X::Attack),
                    CombatStyle::new(S::Swipe, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Fend, A::Crush, St::Defensive, X::Defence),
                    CombatStyle::new(S::Spell, A::Magic, St::Accurate, X::Magic),
                    CombatStyle::new(S::Spell, A::Magic, St::Defensive, X::MagicAndDefence),
                ];
                V
            }
            Self::PoweredStaff => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Accurate, A::Magic, St::Accurate, X::Magic),
                    CombatStyle::new(S::Accurate, A::Magic, St::Accurate, X::Magic),
                    CombatStyle::new(S::Longrange, A::Magic, St::Longrange, X::MagicAndDefence),
                ];
                V
            }
            Self::Salamander => {
                const V: &[CombatStyle] = &[
                    CombatStyle::new(S::Scorch, A::Slash, St::Aggressive, X::Strength),
                    CombatStyle::new(S::Flare, A::Ranged, St::Accurate, X::Ranged),
                    CombatStyle::new(S::Blaze, A::Magic, St::Defensive, X::Defence),
                ];
                V
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(usize)]
pub enum WearPos {
    Head = 0,
    Cape = 1,
    Amulet = 2,
    Weapon = 3,
    Body = 4,
    Shield = 5,
    Legs = 7,
    Gloves = 9,
    Boots = 10,
    Ring = 12,
    Ammo = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WearFlag {
    #[default]
    None,
    TwoHanded,
    Sleeveless,
    Hair,
    HairMid,
    HairLow,
    FullFace,
    Mask,
}

#[derive(Debug, Clone)]
pub struct ObjType {
    pub id: u32,
    pub name: String,
    pub inventory_model: u32,
    pub zoom_2d: u16,
    pub rotation_x: u16,
    pub rotation_y: u16,
    pub offset_x: i16,
    pub offset_y: i16,
    pub rotation_z: u16,
    pub value: i32,
    pub stackable: bool,
    pub members: bool,
    pub stock_market: bool,
    pub ground_options: [Option<String>; 5],
    pub inventory_options: [Option<String>; 5],
    pub male_worn_models: [Option<u32>; 3],
    pub female_worn_models: [Option<u32>; 3],
    pub male_head_model: Option<u32>,
    pub female_head_model: Option<u32>,
    pub noted_id: Option<u32>,
    pub noted_template: Option<u32>,
    pub stack_variants: Vec<(u32, u16)>,
    pub recolor_find: Vec<u16>,
    pub recolor_replace: Vec<u16>,
    pub retexture_find: Vec<u16>,
    pub retexture_replace: Vec<u16>,
    pub team: u8,
    pub weight: i32,
    pub wearpos: Option<WearPos>,
    pub wearflag: WearFlag,
    pub equip: EquipBonuses,
    pub atk_speed: Option<i16>,
    pub atk_seq: Vec<u16>,
    pub block_seq: Option<u16>,
    pub weapon_category: Option<WeaponCategory>,
    pub lent_id: Option<u32>,
    pub lent_template: Option<u32>,
    pub params: HashMap<u32, ParamValue>,
}

impl Default for ObjType {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            inventory_model: 0,
            zoom_2d: 2000,
            rotation_x: 0,
            rotation_y: 0,
            offset_x: 0,
            offset_y: 0,
            rotation_z: 0,
            value: 1,
            stackable: false,
            members: false,
            stock_market: false,
            ground_options: [None, None, Some("Take".to_string()), None, None],
            inventory_options: [None, None, None, None, Some("Drop".to_string())],
            male_worn_models: [None, None, None],
            female_worn_models: [None, None, None],
            male_head_model: None,
            female_head_model: None,
            noted_id: None,
            noted_template: None,
            stack_variants: Vec::new(),
            recolor_find: Vec::new(),
            recolor_replace: Vec::new(),
            retexture_find: Vec::new(),
            retexture_replace: Vec::new(),
            team: 0,
            weight: 0,
            wearpos: None,
            wearflag: WearFlag::None,
            equip: EquipBonuses::default(),
            atk_speed: None,
            atk_seq: Vec::new(),
            block_seq: None,
            weapon_category: None,
            lent_id: None,
            lent_template: None,
            params: HashMap::new(),
        }
    }
}

impl ObjType {
    pub fn decode(id: u32, data: &[u8]) -> anyhow::Result<Self> {
        let mut t = Self {
            id,
            ..Default::default()
        };

        let mut buf = Bytes::copy_from_slice(data);
        loop {
            let opcode = buf.get_u8();
            if opcode == 0 {
                break;
            }
            t.decode_opcode(&mut buf, opcode)?;
        }

        Ok(t)
    }

    fn decode_opcode(&mut self, buf: &mut Bytes, opcode: u8) -> anyhow::Result<()> {
        match opcode {
            1 => {
                self.inventory_model = buf.get_u16() as u32;
            }
            2 => {
                self.name = buf.get_string();
            }
            4 => {
                self.zoom_2d = buf.get_u16();
            }
            5 => {
                self.rotation_x = buf.get_u16();
            }
            6 => {
                self.rotation_y = buf.get_u16();
            }
            7 => {
                self.offset_x = buf.get_u16() as i16;
            }
            8 => {
                self.offset_y = buf.get_u16() as i16;
            }
            11 => {
                self.stackable = true;
            }
            12 => {
                self.value = buf.get_i32();
            }
            16 => {
                self.members = true;
            }
            23 => {
                self.male_worn_models[0] = Some(buf.get_u16() as u32);
            }
            24 => {
                self.male_worn_models[1] = Some(buf.get_u16() as u32);
            }
            25 => {
                self.female_worn_models[0] = Some(buf.get_u16() as u32);
            }
            26 => {
                self.female_worn_models[1] = Some(buf.get_u16() as u32);
            }
            30..=34 => {
                let idx = (opcode - 30) as usize;
                let option = buf.get_string();
                self.ground_options[idx] = if option == "Hidden" { None } else { Some(option) };
            }
            35..=39 => {
                let idx = (opcode - 35) as usize;
                let option = buf.get_string();
                self.inventory_options[idx] = Some(option);
            }
            40 => {
                let count = buf.get_u8() as usize;
                self.recolor_find = Vec::with_capacity(count);
                self.recolor_replace = Vec::with_capacity(count);
                for _ in 0..count {
                    self.recolor_find.push(buf.get_u16());
                    self.recolor_replace.push(buf.get_u16());
                }
            }
            41 => {
                let count = buf.get_u8() as usize;
                self.retexture_find = Vec::with_capacity(count);
                self.retexture_replace = Vec::with_capacity(count);
                for _ in 0..count {
                    self.retexture_find.push(buf.get_u16());
                    self.retexture_replace.push(buf.get_u16());
                }
            }
            42 => {
                let count = buf.get_u8() as usize;
                for _ in 0..count {
                    let _ = buf.get_u8();
                }
            }
            65 => {
                self.stock_market = true;
            }
            78 => {
                self.male_worn_models[2] = Some(buf.get_u16() as u32);
            }
            79 => {
                self.female_worn_models[2] = Some(buf.get_u16() as u32);
            }
            90 => {
                self.male_head_model = Some(buf.get_u16() as u32);
            }
            91 => {
                self.female_head_model = Some(buf.get_u16() as u32);
            }
            92 => {
                let _model = buf.get_u16();
            }
            93 => {
                let _model = buf.get_u16();
            }
            95 => {
                self.rotation_z = buf.get_u16();
            }
            96 => {
                let _ = buf.get_u8();
            }
            97 => {
                self.noted_id = Some(buf.get_u16() as u32);
            }
            98 => {
                self.noted_template = Some(buf.get_u16() as u32);
            }
            100..=109 => {
                let idx = (opcode - 100) as usize;
                if idx >= self.stack_variants.len() {
                    self.stack_variants.resize(idx + 1, (0, 0));
                }
                let variant_id = buf.get_u16() as u32;
                let variant_amount = buf.get_u16();
                self.stack_variants[idx] = (variant_id, variant_amount);
            }
            110 => {
                let _scale = buf.get_u16();
            }
            111 => {
                let _scale = buf.get_u16();
            }
            112 => {
                let _scale = buf.get_u16();
            }
            113 => {
                let _ambient = buf.get_i8();
            }
            114 => {
                let _contrast = buf.get_i8();
            }
            115 => {
                self.team = buf.get_u8();
            }
            121 => {
                self.lent_id = Some(buf.get_u16() as u32);
            }
            122 => {
                self.lent_template = Some(buf.get_u16() as u32);
            }
            125 => {
                let _x = buf.get_i8();
                let _y = buf.get_i8();
                let _z = buf.get_i8();
            }
            126 => {
                let _x = buf.get_i8();
                let _y = buf.get_i8();
                let _z = buf.get_i8();
            }
            127 => {
                let _cursor = buf.get_u8();
                let _index = buf.get_u16();
            }
            128 => {
                let _cursor = buf.get_u8();
                let _index = buf.get_u16();
            }
            129 => {
                let _cursor = buf.get_u8();
                let _index = buf.get_u16();
            }
            130 => {
                let _cursor = buf.get_u8();
                let _index = buf.get_u16();
            }
            132 => {
                let count = buf.get_u8() as usize;
                for _ in 0..count {
                    let _quest_id = buf.get_u16();
                }
            }
            139 => {
                let _ = buf.get_u16();
            }
            140 => {
                let _ = buf.get_u16();
            }
            249 => {
                let count = buf.get_u8() as usize;
                for _ in 0..count {
                    let is_string = buf.get_u8() == 1;
                    let key = buf.get_u24();
                    let value =
                        if is_string { ParamValue::String(buf.get_string()) } else { ParamValue::Int(buf.get_i32()) };
                    self.params.insert(key, value);
                }
            }
            _ => {
                // Unknown opcode
            }
        }

        Ok(())
    }

    pub fn pending_transforms(&self) -> impl Iterator<Item = (TransformKind, u32)> {
        self.noted_template
            .as_ref()
            .zip(self.noted_id)
            .map(|(_, id)| (TransformKind::Noted, id))
            .into_iter()
            .chain(
                self.lent_template
                    .as_ref()
                    .zip(self.lent_id)
                    .map(|(_, id)| (TransformKind::Lent, id)),
            )
    }

    pub fn apply_transform(&mut self, kind: TransformKind, source: &ObjType) {
        match kind {
            TransformKind::Noted => self.transform_noted(source),
            TransformKind::Lent => self.transform_lent(source),
        }
    }

    fn transform_noted(&mut self, noted: &ObjType) {
        self.members = noted.members;
        self.value = noted.value;
        self.name = noted.name.clone();
        self.stackable = true;
        self.params = noted.params.clone();
    }

    fn transform_lent(&mut self, lent: &ObjType) {
        self.recolor_find = lent.recolor_find.clone();
        self.male_worn_models = lent.male_worn_models;
        self.female_worn_models = lent.female_worn_models;
        self.team = lent.team;
        self.value = 0;
        self.members = lent.members;
        self.name = lent.name.clone();
        self.inventory_options = lent.inventory_options.clone();
        self.inventory_options[4] = Some("Discard".to_string());
        self.params = lent.params.clone();
    }
}
