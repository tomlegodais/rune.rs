use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(NpcConfigs::Table)
                    .add_column(
                        ColumnDef::new(NpcConfigs::AtkRange)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(NpcConfigs::Table)
                    .drop_column(NpcConfigs::AtkRange)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum NpcConfigs {
    Table,
    AtkRange,
}
