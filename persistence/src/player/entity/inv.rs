use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvEntry {
    pub obj_id: u16,
    pub amount: u32,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "player_inv")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub player_id: i64,
    #[sea_orm(column_type = "JsonBinary")]
    pub objs: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::player::Entity",
        from = "Column::PlayerId",
        to = "super::player::Column::Id"
    )]
    Player,
}

impl Related<super::player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Player.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
