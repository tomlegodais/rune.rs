use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjAmmoConfigs::Table)
                    .add_column(ColumnDef::new(ObjAmmoConfigs::AtkSpotanim).small_integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjRangedConfigs::Table)
                    .add_column(ColumnDef::new(ObjRangedConfigs::AtkSpotanim).small_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjRangedConfigs::Table)
                    .drop_column(ObjRangedConfigs::AtkSpotanim)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjAmmoConfigs::Table)
                    .drop_column(ObjAmmoConfigs::AtkSpotanim)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ObjAmmoConfigs {
    Table,
    AtkSpotanim,
}

#[derive(DeriveIden)]
enum ObjRangedConfigs {
    Table,
    AtkSpotanim,
}
