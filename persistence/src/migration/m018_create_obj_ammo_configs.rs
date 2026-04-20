use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ObjAmmoConfigs::Table)
                    .if_not_exists()
                    .col(integer(ObjAmmoConfigs::ObjId).primary_key().not_null())
                    .col(
                        ColumnDef::new(ObjAmmoConfigs::AmmoType)
                            .custom(AmmoType::Type)
                            .not_null(),
                    )
                    .col(small_integer(ObjAmmoConfigs::AmmoTier).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ObjAmmoConfigs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ObjAmmoConfigs {
    Table,
    ObjId,
    AmmoType,
    AmmoTier,
}

#[derive(DeriveIden)]
enum AmmoType {
    #[sea_orm(iden = "ammo_type")]
    Type,
}
