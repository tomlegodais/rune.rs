use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(FaceDirection::Type)
                    .values([
                        FaceDirection::North,
                        FaceDirection::NorthEast,
                        FaceDirection::East,
                        FaceDirection::SouthEast,
                        FaceDirection::South,
                        FaceDirection::SouthWest,
                        FaceDirection::West,
                        FaceDirection::NorthWest,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NpcConfigs::Table)
                    .if_not_exists()
                    .col(integer(NpcConfigs::NpcId).primary_key().not_null())
                    .col(integer(NpcConfigs::MaxHp).not_null().default(1))
                    .col(small_integer(NpcConfigs::AtkLevel).not_null().default(1))
                    .col(small_integer(NpcConfigs::StrLevel).not_null().default(1))
                    .col(small_integer(NpcConfigs::DefLevel).not_null().default(1))
                    .col(small_integer(NpcConfigs::AtkBonus).not_null().default(0))
                    .col(small_integer(NpcConfigs::StrBonus).not_null().default(0))
                    .col(small_integer(NpcConfigs::DefStab).not_null().default(0))
                    .col(small_integer(NpcConfigs::DefSlash).not_null().default(0))
                    .col(small_integer(NpcConfigs::DefCrush).not_null().default(0))
                    .col(small_integer(NpcConfigs::DefMagic).not_null().default(0))
                    .col(small_integer(NpcConfigs::DefRanged).not_null().default(0))
                    .col(small_integer(NpcConfigs::AtkSpeed).not_null().default(4))
                    .col(small_integer(NpcConfigs::AtkSeq).not_null().default(422))
                    .col(small_integer(NpcConfigs::BlockSeq).not_null().default(424))
                    .col(small_integer(NpcConfigs::DeathSeq).not_null().default(836))
                    .col(small_integer(NpcConfigs::MaxHit).not_null().default(1))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NpcSpawns::Table)
                    .if_not_exists()
                    .col(integer(NpcSpawns::Id).primary_key().auto_increment().not_null())
                    .col(integer(NpcSpawns::NpcId).not_null())
                    .col(integer(NpcSpawns::X).not_null())
                    .col(integer(NpcSpawns::Y).not_null())
                    .col(integer(NpcSpawns::Plane).not_null().default(0))
                    .col(small_integer(NpcSpawns::WanderRadius).not_null().default(0))
                    .col(small_integer(NpcSpawns::RespawnTicks).not_null().default(50))
                    .col(
                        ColumnDef::new(NpcSpawns::FaceDirection)
                            .custom(FaceDirection::Type)
                            .not_null()
                            .default("south"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NpcSpawns::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NpcConfigs::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(FaceDirection::Type).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FaceDirection {
    #[sea_orm(iden = "face_direction")]
    Type,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(DeriveIden)]
enum NpcConfigs {
    Table,
    NpcId,
    MaxHp,
    AtkLevel,
    StrLevel,
    DefLevel,
    AtkBonus,
    StrBonus,
    DefStab,
    DefSlash,
    DefCrush,
    DefMagic,
    DefRanged,
    AtkSpeed,
    AtkSeq,
    BlockSeq,
    DeathSeq,
    MaxHit,
}

#[derive(DeriveIden)]
enum NpcSpawns {
    Table,
    Id,
    NpcId,
    X,
    Y,
    Plane,
    WanderRadius,
    RespawnTicks,
    FaceDirection,
}
