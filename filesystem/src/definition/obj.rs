use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use crate::definition::ParamValue;

pub enum TransformKind {
    Noted,
    Lent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(usize)]
pub enum EquipmentSlot {
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
pub enum EquipmentFlag {
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
    pub male_equip_models: [Option<u32>; 3],
    pub female_equip_models: [Option<u32>; 3],
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
    pub equipment_slot: Option<EquipmentSlot>,
    pub equipment_flag: EquipmentFlag,
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
            male_equip_models: [None, None, None],
            female_equip_models: [None, None, None],
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
            equipment_slot: None,
            equipment_flag: EquipmentFlag::None,
            lent_id: None,
            lent_template: None,
            params: HashMap::new(),
        }
    }
}

impl ObjType {
    pub fn decode(id: u32, data: &[u8]) -> anyhow::Result<Self> {
        let mut def = Self {
            id,
            ..Default::default()
        };

        let mut buf = Bytes::copy_from_slice(data);
        loop {
            let opcode = buf.get_u8();
            if opcode == 0 {
                break;
            }
            def.decode_opcode(&mut buf, opcode)?;
        }

        Ok(def)
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
                self.male_equip_models[0] = Some(buf.get_u16() as u32);
            }
            24 => {
                self.male_equip_models[1] = Some(buf.get_u16() as u32);
            }
            25 => {
                self.female_equip_models[0] = Some(buf.get_u16() as u32);
            }
            26 => {
                self.female_equip_models[1] = Some(buf.get_u16() as u32);
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
                self.male_equip_models[2] = Some(buf.get_u16() as u32);
            }
            79 => {
                self.female_equip_models[2] = Some(buf.get_u16() as u32);
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

    fn transform_noted(&mut self, noted_def: &ObjType) {
        self.members = noted_def.members;
        self.value = noted_def.value;
        self.name = noted_def.name.clone();
        self.stackable = true;
        self.params = noted_def.params.clone();
    }

    fn transform_lent(&mut self, lent_def: &ObjType) {
        self.recolor_find = lent_def.recolor_find.clone();
        self.male_equip_models = lent_def.male_equip_models;
        self.female_equip_models = lent_def.female_equip_models;
        self.team = lent_def.team;
        self.value = 0;
        self.members = lent_def.members;
        self.name = lent_def.name.clone();
        self.inventory_options = lent_def.inventory_options.clone();
        self.inventory_options[4] = Some("Discard".to_string());
        self.params = lent_def.params.clone();
    }
}
