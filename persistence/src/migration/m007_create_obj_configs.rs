use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(WearPos::Type)
                    .values([
                        WearPos::Head,
                        WearPos::Cape,
                        WearPos::Amulet,
                        WearPos::Weapon,
                        WearPos::Body,
                        WearPos::Shield,
                        WearPos::Legs,
                        WearPos::Gloves,
                        WearPos::Boots,
                        WearPos::Ring,
                        WearPos::Ammo,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(WearFlag::Type)
                    .values([
                        WearFlag::TwoHanded,
                        WearFlag::Sleeveless,
                        WearFlag::Hair,
                        WearFlag::HairMid,
                        WearFlag::HairLow,
                        WearFlag::FullFace,
                        WearFlag::Mask,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ObjConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjConfigs::ObjId).primary_key().not_null())
                    .col(ColumnDef::new(ObjConfigs::WearPos).custom(WearPos::Type).null())
                    .col(ColumnDef::new(ObjConfigs::WearFlag).custom(WearFlag::Type).null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ObjConfigs::Table).to_owned())
            .await?;

        manager.drop_type(Type::drop().name(WearFlag::Type).to_owned()).await?;

        manager.drop_type(Type::drop().name(WearPos::Type).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum ObjConfigs {
    Table,
    ObjId,
    WearPos,
    WearFlag,
}

#[derive(DeriveIden)]
enum WearPos {
    #[sea_orm(iden = "wearpos")]
    Type,
    Head,
    Cape,
    Amulet,
    Weapon,
    Body,
    Shield,
    Legs,
    Gloves,
    Boots,
    Ring,
    Ammo,
}

#[derive(DeriveIden)]
enum WearFlag {
    #[sea_orm(iden = "wearflag")]
    Type,
    TwoHanded,
    Sleeveless,
    Hair,
    HairMid,
    HairLow,
    FullFace,
    Mask,
}
