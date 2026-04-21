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
                    .rename_column(ObjAmmoConfigs::ProjGfx, ObjAmmoConfigs::ProjSpotanim)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjRangedConfigs::Table)
                    .rename_column(ObjRangedConfigs::ProjGfx, ObjRangedConfigs::ProjSpotanim)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjRangedConfigs::Table)
                    .rename_column(ObjRangedConfigs::ProjSpotanim, ObjRangedConfigs::ProjGfx)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjAmmoConfigs::Table)
                    .rename_column(ObjAmmoConfigs::ProjSpotanim, ObjAmmoConfigs::ProjGfx)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ObjAmmoConfigs {
    Table,
    ProjGfx,
    ProjSpotanim,
}

#[derive(DeriveIden)]
enum ObjRangedConfigs {
    Table,
    ProjGfx,
    ProjSpotanim,
}
