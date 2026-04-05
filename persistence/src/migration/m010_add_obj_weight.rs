use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .add_column(integer(ObjConfigs::Weight).not_null().default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .drop_column(ObjConfigs::Weight)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ObjConfigs {
    Table,
    Weight,
}
