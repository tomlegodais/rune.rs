mod item;
mod loc;
mod npc;
mod varbit;

pub use item::{ItemDefinition, TransformKind};
pub use loc::LocDefinition;
pub use npc::NpcDefinition;
pub use varbit::VarbitDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Int(i32),
    String(String),
}
