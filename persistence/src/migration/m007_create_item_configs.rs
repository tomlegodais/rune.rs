use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(EquipmentSlot::Type)
                    .values([
                        EquipmentSlot::Head,
                        EquipmentSlot::Cape,
                        EquipmentSlot::Amulet,
                        EquipmentSlot::Weapon,
                        EquipmentSlot::Body,
                        EquipmentSlot::Shield,
                        EquipmentSlot::Legs,
                        EquipmentSlot::Gloves,
                        EquipmentSlot::Boots,
                        EquipmentSlot::Ring,
                        EquipmentSlot::Ammo,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(EquipmentFlag::Type)
                    .values([
                        EquipmentFlag::TwoHanded,
                        EquipmentFlag::Sleeveless,
                        EquipmentFlag::Hair,
                        EquipmentFlag::HairMid,
                        EquipmentFlag::HairLow,
                        EquipmentFlag::FullFace,
                        EquipmentFlag::Mask,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ItemConfigs::Table)
                    .if_not_exists()
                    .col(integer(ItemConfigs::ItemId).primary_key().not_null())
                    .col(
                        ColumnDef::new(ItemConfigs::EquipmentSlot)
                            .custom(EquipmentSlot::Type)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ItemConfigs::EquipmentFlag)
                            .custom(EquipmentFlag::Type)
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ItemConfigs::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(EquipmentFlag::Type).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(EquipmentSlot::Type).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ItemConfigs {
    Table,
    ItemId,
    EquipmentSlot,
    EquipmentFlag,
}

#[derive(DeriveIden)]
enum EquipmentSlot {
    #[sea_orm(iden = "equipment_slot")]
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
enum EquipmentFlag {
    #[sea_orm(iden = "equipment_flag")]
    Type,
    TwoHanded,
    Sleeveless,
    Hair,
    HairMid,
    HairLow,
    FullFace,
    Mask,
}
