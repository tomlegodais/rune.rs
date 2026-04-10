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
                        ColumnDef::new(Players::CombatStyle)
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(Players::AutoRetaliate)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .add_column(
                        ColumnDef::new(Players::SpecEnergy)
                            .small_integer()
                            .not_null()
                            .default(1000),
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
                    .drop_column(Players::CombatStyle)
                    .drop_column(Players::AutoRetaliate)
                    .drop_column(Players::SpecEnergy)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Players {
    Table,
    CombatStyle,
    AutoRetaliate,
    SpecEnergy,
}
