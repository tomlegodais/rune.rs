mod enums;
mod loc;
mod npc;
mod obj;
mod structs;
mod varbit;

pub use enums::{EnumType, EnumValue};
pub use loc::LocType;
pub use npc::NpcType;
pub use obj::{ObjType, TransformKind, WearFlag, WearPos};
pub use structs::StructType;
pub use varbit::VarbitType;

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
