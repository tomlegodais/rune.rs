use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Players::Table)
                    .add_column(
                        ColumnDef::new(Players::RunEnergy)
                            .small_integer()
                            .not_null()
                            .default(10000),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Players::Table)
                    .drop_column(Players::RunEnergy)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Players {
    Table,
    RunEnergy,
}
