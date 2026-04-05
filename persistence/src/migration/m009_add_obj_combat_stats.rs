use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(WeaponCategory::Type)
                    .values([
                        WeaponCategory::TwoHandedSword,
                        WeaponCategory::Axe,
                        WeaponCategory::Banner,
                        WeaponCategory::Blunt,
                        WeaponCategory::Bludgeon,
                        WeaponCategory::Bulwark,
                        WeaponCategory::Claw,
                        WeaponCategory::Egg,
                        WeaponCategory::Partisan,
                        WeaponCategory::Pickaxe,
                        WeaponCategory::Polearm,
                        WeaponCategory::Polestaff,
                        WeaponCategory::Scythe,
                        WeaponCategory::SlashSword,
                        WeaponCategory::Spear,
                        WeaponCategory::Spiked,
                        WeaponCategory::StabSword,
                        WeaponCategory::Unarmed,
                        WeaponCategory::Whip,
                        WeaponCategory::Bow,
                        WeaponCategory::Blaster,
                        WeaponCategory::Chinchompa,
                        WeaponCategory::Crossbow,
                        WeaponCategory::Gun,
                        WeaponCategory::Thrown,
                        WeaponCategory::BladedStaff,
                        WeaponCategory::PoweredStaff,
                        WeaponCategory::Staff,
                        WeaponCategory::Salamander,
                        WeaponCategory::MultiStyle,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .add_column(ColumnDef::new(ObjConfigs::WeaponCategory).custom(WeaponCategory::Type).null())
                    .add_column(small_integer(ObjConfigs::AtkStab).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::AtkSlash).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::AtkCrush).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::AtkMagic).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::AtkRanged).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::DefStab).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::DefSlash).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::DefCrush).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::DefMagic).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::DefRanged).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::StrBonus).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::RangedStr).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::MagicDmg).not_null().default(0))
                    .add_column(small_integer(ObjConfigs::Prayer).not_null().default(0))
                    .add_column(ColumnDef::new(ObjConfigs::AtkSpeed).small_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .drop_column(ObjConfigs::WeaponCategory)
                    .drop_column(ObjConfigs::AtkStab)
                    .drop_column(ObjConfigs::AtkSlash)
                    .drop_column(ObjConfigs::AtkCrush)
                    .drop_column(ObjConfigs::AtkMagic)
                    .drop_column(ObjConfigs::AtkRanged)
                    .drop_column(ObjConfigs::DefStab)
                    .drop_column(ObjConfigs::DefSlash)
                    .drop_column(ObjConfigs::DefCrush)
                    .drop_column(ObjConfigs::DefMagic)
                    .drop_column(ObjConfigs::DefRanged)
                    .drop_column(ObjConfigs::StrBonus)
                    .drop_column(ObjConfigs::RangedStr)
                    .drop_column(ObjConfigs::MagicDmg)
                    .drop_column(ObjConfigs::Prayer)
                    .drop_column(ObjConfigs::AtkSpeed)
                    .to_owned(),
            )
            .await?;

        manager.drop_type(Type::drop().name(WeaponCategory::Type).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum ObjConfigs {
    Table,
    WeaponCategory,
    AtkStab,
    AtkSlash,
    AtkCrush,
    AtkMagic,
    AtkRanged,
    DefStab,
    DefSlash,
    DefCrush,
    DefMagic,
    DefRanged,
    StrBonus,
    RangedStr,
    MagicDmg,
    Prayer,
    AtkSpeed,
}

#[derive(DeriveIden)]
enum WeaponCategory {
    #[sea_orm(iden = "weapon_category")]
    Type,
    TwoHandedSword,
    Axe,
    Banner,
    Blunt,
    Bludgeon,
    Bulwark,
    Claw,
    Egg,
    Partisan,
    Pickaxe,
    Polearm,
    Polestaff,
    Scythe,
    SlashSword,
    Spear,
    Spiked,
    StabSword,
    Unarmed,
    Whip,
    Bow,
    Blaster,
    Chinchompa,
    Crossbow,
    Gun,
    Thrown,
    BladedStaff,
    PoweredStaff,
    Staff,
    Salamander,
    MultiStyle,
}
