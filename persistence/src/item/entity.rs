use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i32,
    pub equipment_slot: Option<i16>,
    pub two_handed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
