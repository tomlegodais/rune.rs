use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .add_column(
                        ColumnDef::new(ObjConfigs::AtkSeq)
                            .array(sea_orm_migration::sea_query::ColumnType::SmallInteger)
                            .null(),
                    )
                    .add_column(ColumnDef::new(ObjConfigs::BlockSeq).small_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .drop_column(ObjConfigs::AtkSeq)
                    .drop_column(ObjConfigs::BlockSeq)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ObjConfigs {
    Table,
    AtkSeq,
    BlockSeq,
}
