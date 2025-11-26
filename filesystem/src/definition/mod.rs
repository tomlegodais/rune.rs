mod item;

pub use item::ItemDefinition;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    Int(i32),
    String(String),
}
