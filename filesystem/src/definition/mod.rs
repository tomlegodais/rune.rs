mod enums;
mod item;
mod loc;
mod npc;
mod structs;
mod varbit;

pub use enums::{EnumDefinition, EnumValue};
pub use item::{EquipmentFlag, EquipmentSlot, ItemDefinition, TransformKind};
pub use loc::LocDefinition;
pub use npc::NpcDefinition;
pub use structs::StructDefinition;
pub use varbit::VarbitDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Int(i32),
    String(String),
}

pub trait ParamMap {
    fn int_param(&self, key: u32) -> Option<i32>;
    fn str_param(&self, key: u32) -> Option<&str>;
}

impl ParamMap for std::collections::HashMap<u32, ParamValue> {
    fn int_param(&self, key: u32) -> Option<i32> {
        match self.get(&key)? {
            ParamValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    fn str_param(&self, key: u32) -> Option<&str> {
        match self.get(&key)? {
            ParamValue::String(v) => Some(v),
            _ => None,
        }
    }
}
