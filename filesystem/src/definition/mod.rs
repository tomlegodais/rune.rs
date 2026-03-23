mod item;
mod loc;

pub use item::ItemDefinition;
pub use loc::LocDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Int(i32),
    String(String),
}
