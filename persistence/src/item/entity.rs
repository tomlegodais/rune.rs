use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "equipment_slot")]
pub enum EquipmentSlot {
    #[sea_orm(string_value = "head")]
    Head,
    #[sea_orm(string_value = "cape")]
    Cape,
    #[sea_orm(string_value = "amulet")]
    Amulet,
    #[sea_orm(string_value = "weapon")]
    Weapon,
    #[sea_orm(string_value = "body")]
    Body,
    #[sea_orm(string_value = "shield")]
    Shield,
    #[sea_orm(string_value = "legs")]
    Legs,
    #[sea_orm(string_value = "gloves")]
    Gloves,
    #[sea_orm(string_value = "boots")]
    Boots,
    #[sea_orm(string_value = "ring")]
    Ring,
    #[sea_orm(string_value = "ammo")]
    Ammo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "equipment_flag")]
pub enum EquipmentFlag {
    #[sea_orm(string_value = "two_handed")]
    TwoHanded,
    #[sea_orm(string_value = "sleeveless")]
    Sleeveless,
    #[sea_orm(string_value = "hair")]
    Hair,
    #[sea_orm(string_value = "hair_mid")]
    HairMid,
    #[sea_orm(string_value = "hair_low")]
    HairLow,
    #[sea_orm(string_value = "full_face")]
    FullFace,
    #[sea_orm(string_value = "mask")]
    Mask,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i32,
    pub equipment_slot: Option<EquipmentSlot>,
    pub equipment_flag: Option<EquipmentFlag>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
