use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "weapon_category")]
pub enum WeaponCategory {
    #[sea_orm(string_value = "two_handed_sword")]
    TwoHandedSword,
    #[sea_orm(string_value = "axe")]
    Axe,
    #[sea_orm(string_value = "banner")]
    Banner,
    #[sea_orm(string_value = "blunt")]
    Blunt,
    #[sea_orm(string_value = "bludgeon")]
    Bludgeon,
    #[sea_orm(string_value = "bulwark")]
    Bulwark,
    #[sea_orm(string_value = "claw")]
    Claw,
    #[sea_orm(string_value = "egg")]
    Egg,
    #[sea_orm(string_value = "partisan")]
    Partisan,
    #[sea_orm(string_value = "pickaxe")]
    Pickaxe,
    #[sea_orm(string_value = "polearm")]
    Polearm,
    #[sea_orm(string_value = "polestaff")]
    Polestaff,
    #[sea_orm(string_value = "scythe")]
    Scythe,
    #[sea_orm(string_value = "slash_sword")]
    SlashSword,
    #[sea_orm(string_value = "spear")]
    Spear,
    #[sea_orm(string_value = "spiked")]
    Spiked,
    #[sea_orm(string_value = "stab_sword")]
    StabSword,
    #[sea_orm(string_value = "unarmed")]
    Unarmed,
    #[sea_orm(string_value = "whip")]
    Whip,
    #[sea_orm(string_value = "bow")]
    Bow,
    #[sea_orm(string_value = "blaster")]
    Blaster,
    #[sea_orm(string_value = "chinchompa")]
    Chinchompa,
    #[sea_orm(string_value = "crossbow")]
    Crossbow,
    #[sea_orm(string_value = "gun")]
    Gun,
    #[sea_orm(string_value = "thrown")]
    Thrown,
    #[sea_orm(string_value = "bladed_staff")]
    BladedStaff,
    #[sea_orm(string_value = "powered_staff")]
    PoweredStaff,
    #[sea_orm(string_value = "staff")]
    Staff,
    #[sea_orm(string_value = "salamander")]
    Salamander,
    #[sea_orm(string_value = "multi_style")]
    MultiStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "wearpos")]
pub enum WearPos {
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
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "wearflag")]
pub enum WearFlag {
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
#[sea_orm(table_name = "obj_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub obj_id: i32,
    #[sea_orm(column_name = "wear_pos")]
    pub wearpos: Option<WearPos>,
    #[sea_orm(column_name = "wear_flag")]
    pub wearflag: Option<WearFlag>,
    pub weapon_category: Option<WeaponCategory>,
    pub atk_stab: i16,
    pub atk_slash: i16,
    pub atk_crush: i16,
    pub atk_magic: i16,
    pub atk_ranged: i16,
    pub def_stab: i16,
    pub def_slash: i16,
    pub def_crush: i16,
    pub def_magic: i16,
    pub def_ranged: i16,
    pub str_bonus: i16,
    pub ranged_str: i16,
    pub magic_dmg: i16,
    pub prayer: i16,
    pub atk_speed: Option<i16>,
    pub weight: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
