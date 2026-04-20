use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ObjStatConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjStatConfigs::ObjId).primary_key().not_null())
                    .col(small_integer(ObjStatConfigs::AtkStab).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::AtkSlash).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::AtkCrush).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::AtkMagic).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::AtkRanged).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::DefStab).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::DefSlash).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::DefCrush).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::DefMagic).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::DefRanged).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::StrBonus).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::RangedStr).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::MagicDmg).not_null().default(0))
                    .col(small_integer(ObjStatConfigs::Prayer).not_null().default(0))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ObjWeaponConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjWeaponConfigs::ObjId).primary_key().not_null())
                    .col(
                        ColumnDef::new(ObjWeaponConfigs::WeaponCategory)
                            .custom(WeaponCategory::Type)
                            .not_null(),
                    )
                    .col(small_integer(ObjWeaponConfigs::AtkSpeed).not_null())
                    .col(
                        ColumnDef::new(ObjWeaponConfigs::AtkSeq)
                            .array(sea_orm_migration::sea_query::ColumnType::SmallInteger)
                            .null(),
                    )
                    .col(ColumnDef::new(ObjWeaponConfigs::BlockSeq).small_integer().null())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(
            "INSERT INTO obj_stat_configs (obj_id, atk_stab, atk_slash, atk_crush, atk_magic, atk_ranged, def_stab, def_slash, def_crush, def_magic, def_ranged, str_bonus, ranged_str, magic_dmg, prayer)
             SELECT obj_id, atk_stab, atk_slash, atk_crush, atk_magic, atk_ranged, def_stab, def_slash, def_crush, def_magic, def_ranged, str_bonus, ranged_str, magic_dmg, prayer FROM obj_combat_configs
             WHERE atk_stab <> 0 OR atk_slash <> 0 OR atk_crush <> 0 OR atk_magic <> 0 OR atk_ranged <> 0
                OR def_stab <> 0 OR def_slash <> 0 OR def_crush <> 0 OR def_magic <> 0 OR def_ranged <> 0
                OR str_bonus <> 0 OR ranged_str <> 0 OR magic_dmg <> 0 OR prayer <> 0",
        )
        .await?;

        db.execute_unprepared(
            "INSERT INTO obj_weapon_configs (obj_id, weapon_category, atk_speed, atk_seq, block_seq)
             SELECT obj_id, weapon_category, atk_speed, atk_seq, block_seq FROM obj_combat_configs
             WHERE weapon_category IS NOT NULL",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(ObjCombatConfigs::Table).to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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

        let db = manager.get_connection();

        db.execute_unprepared(
            "INSERT INTO obj_combat_configs (obj_id, atk_stab, atk_slash, atk_crush, atk_magic, atk_ranged, def_stab, def_slash, def_crush, def_magic, def_ranged, str_bonus, ranged_str, magic_dmg, prayer)
             SELECT obj_id, atk_stab, atk_slash, atk_crush, atk_magic, atk_ranged, def_stab, def_slash, def_crush, def_magic, def_ranged, str_bonus, ranged_str, magic_dmg, prayer FROM obj_stat_configs",
        )
        .await?;

        db.execute_unprepared(
            "UPDATE obj_combat_configs SET
                weapon_category = w.weapon_category, atk_speed = w.atk_speed, atk_seq = w.atk_seq, block_seq = w.block_seq
             FROM obj_weapon_configs w WHERE obj_combat_configs.obj_id = w.obj_id",
        )
        .await?;

        db.execute_unprepared(
            "INSERT INTO obj_combat_configs (obj_id, weapon_category, atk_speed, atk_seq, block_seq)
             SELECT obj_id, weapon_category, atk_speed, atk_seq, block_seq FROM obj_weapon_configs w
             WHERE NOT EXISTS (SELECT 1 FROM obj_combat_configs c WHERE c.obj_id = w.obj_id)",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(ObjWeaponConfigs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ObjStatConfigs::Table).to_owned())
            .await
    }
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
enum ObjStatConfigs {
    Table,
    ObjId,
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
}

#[derive(DeriveIden)]
enum ObjWeaponConfigs {
    Table,
    ObjId,
    WeaponCategory,
    AtkSpeed,
    AtkSeq,
    BlockSeq,
}

#[derive(DeriveIden)]
enum WeaponCategory {
    #[sea_orm(iden = "weapon_category")]
    Type,
}
