use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "players")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub account_id: i64,
    pub x: i32,
    pub y: i32,
    pub plane: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::account::entity::Entity",
        from = "Column::AccountId",
        to = "crate::account::entity::Column::Id"
    )]
    Account,
    #[sea_orm(has_one = "super::appearance::Entity")]
    Appearance,
    #[sea_orm(has_one = "super::skills::Entity")]
    Skills,
}

impl Related<crate::account::entity::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::appearance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Appearance.def()
    }
}

impl Related<super::skills::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Skills.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}