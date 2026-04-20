use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ObjWearConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjWearConfigs::ObjId).primary_key().not_null())
                    .col(ColumnDef::new(ObjWearConfigs::WearPos).custom(WearPos::Type).null())
                    .col(ColumnDef::new(ObjWearConfigs::WearFlag).custom(WearFlag::Type).null())
                    .col(integer(ObjWearConfigs::Weight).not_null().default(0))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ObjCombatConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjCombatConfigs::ObjId).primary_key().not_null())
                    .col(
                        ColumnDef::new(ObjCombatConfigs::WeaponCategory)
                            .custom(WeaponCategory::Type)
                            .null(),
                    )
                    .col(small_integer(ObjCombatConfigs::AtkStab).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::AtkSlash).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::AtkCrush).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::AtkMagic).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::AtkRanged).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::DefStab).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::DefSlash).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::DefCrush).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::DefMagic).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::DefRanged).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::StrBonus).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::RangedStr).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::MagicDmg).not_null().default(0))
                    .col(small_integer(ObjCombatConfigs::Prayer).not_null().default(0))
                    .col(ColumnDef::new(ObjCombatConfigs::AtkSpeed).small_integer().null())
                    .col(
                        ColumnDef::new(ObjCombatConfigs::AtkSeq)
                            .array(sea_orm_migration::sea_query::ColumnType::SmallInteger)
                            .null(),
                    )
                    .col(ColumnDef::new(ObjCombatConfigs::BlockSeq).small_integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ObjRangedConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjRangedConfigs::ObjId).primary_key().not_null())
                    .col(ColumnDef::new(ObjRangedConfigs::AmmoType).custom(AmmoType::Type).null())
                    .col(ColumnDef::new(ObjRangedConfigs::AmmoTier).small_integer().null())
                    .col(ColumnDef::new(ObjRangedConfigs::AtkRange).small_integer().null())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(
            "INSERT INTO obj_wear_configs (obj_id, wear_pos, wear_flag, weight)
             SELECT obj_id, wear_pos, wear_flag, weight FROM obj_configs
             WHERE wear_pos IS NOT NULL OR wear_flag IS NOT NULL OR weight <> 0",
        )
        .await?;

        db.execute_unprepared(
            "INSERT INTO obj_combat_configs (obj_id, weapon_category, atk_stab, atk_slash, atk_crush, atk_magic, atk_ranged, def_stab, def_slash, def_crush, def_magic, def_ranged, str_bonus, ranged_str, magic_dmg, prayer, atk_speed, atk_seq, block_seq)
             SELECT obj_id, weapon_category, atk_stab, atk_slash, atk_crush, atk_magic, atk_ranged, def_stab, def_slash, def_crush, def_magic, def_ranged, str_bonus, ranged_str, magic_dmg, prayer, atk_speed, atk_seq, block_seq FROM obj_configs
             WHERE weapon_category IS NOT NULL OR atk_speed IS NOT NULL OR atk_seq IS NOT NULL OR block_seq IS NOT NULL
                OR atk_stab <> 0 OR atk_slash <> 0 OR atk_crush <> 0 OR atk_magic <> 0 OR atk_ranged <> 0
                OR def_stab <> 0 OR def_slash <> 0 OR def_crush <> 0 OR def_magic <> 0 OR def_ranged <> 0
                OR str_bonus <> 0 OR ranged_str <> 0 OR magic_dmg <> 0 OR prayer <> 0",
        )
        .await?;

        db.execute_unprepared(
            "INSERT INTO obj_ranged_configs (obj_id, ammo_type, ammo_tier, atk_range)
             SELECT obj_id, ammo_type, ammo_tier, atk_range FROM obj_configs
             WHERE ammo_type IS NOT NULL OR ammo_tier IS NOT NULL OR atk_range IS NOT NULL",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(ObjConfigs::Table).to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ObjConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjConfigs::ObjId).primary_key().not_null())
                    .col(ColumnDef::new(ObjConfigs::WearPos).custom(WearPos::Type).null())
                    .col(ColumnDef::new(ObjConfigs::WearFlag).custom(WearFlag::Type).null())
                    .col(
                        ColumnDef::new(ObjConfigs::WeaponCategory)
                            .custom(WeaponCategory::Type)
                            .null(),
                    )
                    .col(small_integer(ObjConfigs::AtkStab).not_null().default(0))
                    .col(small_integer(ObjConfigs::AtkSlash).not_null().default(0))
                    .col(small_integer(ObjConfigs::AtkCrush).not_null().default(0))
                    .col(small_integer(ObjConfigs::AtkMagic).not_null().default(0))
                    .col(small_integer(ObjConfigs::AtkRanged).not_null().default(0))
                    .col(small_integer(ObjConfigs::DefStab).not_null().default(0))
                    .col(small_integer(ObjConfigs::DefSlash).not_null().default(0))
                    .col(small_integer(ObjConfigs::DefCrush).not_null().default(0))
                    .col(small_integer(ObjConfigs::DefMagic).not_null().default(0))
                    .col(small_integer(ObjConfigs::DefRanged).not_null().default(0))
                    .col(small_integer(ObjConfigs::StrBonus).not_null().default(0))
                    .col(small_integer(ObjConfigs::RangedStr).not_null().default(0))
                    .col(small_integer(ObjConfigs::MagicDmg).not_null().default(0))
                    .col(small_integer(ObjConfigs::Prayer).not_null().default(0))
                    .col(ColumnDef::new(ObjConfigs::AtkSpeed).small_integer().null())
                    .col(integer(ObjConfigs::Weight).not_null().default(0))
                    .col(
                        ColumnDef::new(ObjConfigs::AtkSeq)
                            .array(sea_orm_migration::sea_query::ColumnType::SmallInteger)
                            .null(),
                    )
                    .col(ColumnDef::new(ObjConfigs::BlockSeq).small_integer().null())
                    .col(ColumnDef::new(ObjConfigs::AmmoType).custom(AmmoType::Type).null())
                    .col(ColumnDef::new(ObjConfigs::AmmoTier).small_integer().null())
                    .col(ColumnDef::new(ObjConfigs::AtkRange).small_integer().null())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(
            "INSERT INTO obj_configs (obj_id, wear_pos, wear_flag, weight)
             SELECT obj_id, wear_pos, wear_flag, weight FROM obj_wear_configs",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE obj_configs SET
                weapon_category = c.weapon_category, atk_stab = c.atk_stab, atk_slash = c.atk_slash,
                atk_crush = c.atk_crush, atk_magic = c.atk_magic, atk_ranged = c.atk_ranged,
                def_stab = c.def_stab, def_slash = c.def_slash, def_crush = c.def_crush,
                def_magic = c.def_magic, def_ranged = c.def_ranged, str_bonus = c.str_bonus,
                ranged_str = c.ranged_str, magic_dmg = c.magic_dmg, prayer = c.prayer,
                atk_speed = c.atk_speed, atk_seq = c.atk_seq, block_seq = c.block_seq
             FROM obj_combat_configs c WHERE obj_configs.obj_id = c.obj_id",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE obj_configs SET
                ammo_type = r.ammo_type, ammo_tier = r.ammo_tier, atk_range = r.atk_range
             FROM obj_ranged_configs r WHERE obj_configs.obj_id = r.obj_id",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(ObjRangedConfigs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ObjCombatConfigs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ObjWearConfigs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ObjConfigs {
    Table,
    ObjId,
    WearPos,
    WearFlag,
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
    Weight,
    AtkSeq,
    BlockSeq,
    AmmoType,
    AmmoTier,
    AtkRange,
}

#[derive(DeriveIden)]
enum ObjWearConfigs {
    Table,
    ObjId,
    WearPos,
    WearFlag,
    Weight,
}

#[derive(DeriveIden)]
enum ObjCombatConfigs {
    Table,
    ObjId,
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
    AtkSeq,
    BlockSeq,
}

#[derive(DeriveIden)]
enum ObjRangedConfigs {
    Table,
    ObjId,
    AmmoType,
    AmmoTier,
    AtkRange,
}

#[derive(DeriveIden)]
enum WearPos {
    #[sea_orm(iden = "wearpos")]
    Type,
}

#[derive(DeriveIden)]
enum WearFlag {
    #[sea_orm(iden = "wearflag")]
    Type,
}

#[derive(DeriveIden)]
enum WeaponCategory {
    #[sea_orm(iden = "weapon_category")]
    Type,
}

#[derive(DeriveIden)]
enum AmmoType {
    #[sea_orm(iden = "ammo_type")]
    Type,
}
