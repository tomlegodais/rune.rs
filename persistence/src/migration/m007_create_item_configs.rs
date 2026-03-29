use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ItemConfigs::Table)
                    .if_not_exists()
                    .col(integer(ItemConfigs::ItemId).primary_key().not_null())
                    .col(small_integer_null(ItemConfigs::EquipmentSlot))
                    .col(boolean(ItemConfigs::TwoHanded).not_null().default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ItemConfigs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ItemConfigs {
    Table,
    ItemId,
    EquipmentSlot,
    TwoHanded,
}
