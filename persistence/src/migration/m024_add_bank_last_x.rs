use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PlayerBank::Table)
                    .add_column(integer(PlayerBank::LastX).not_null().default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PlayerBank::Table)
                    .drop_column(PlayerBank::LastX)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PlayerBank {
    Table,
    LastX,
}
