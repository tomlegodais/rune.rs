use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AmmoType::Type)
                    .values([AmmoType::Arrow, AmmoType::Bolt])
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .add_column(ColumnDef::new(ObjConfigs::AmmoType).custom(AmmoType::Type).null())
                    .add_column(ColumnDef::new(ObjConfigs::AmmoTier).small_integer().null())
                    .add_column(ColumnDef::new(ObjConfigs::AtkRange).small_integer().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ObjConfigs::Table)
                    .drop_column(ObjConfigs::AmmoType)
                    .drop_column(ObjConfigs::AmmoTier)
                    .drop_column(ObjConfigs::AtkRange)
                    .to_owned(),
            )
            .await?;

        manager.drop_type(Type::drop().name(AmmoType::Type).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum AmmoType {
    #[sea_orm(iden = "ammo_type")]
    Type,
    Arrow,
    Bolt,
}

#[derive(DeriveIden)]
enum ObjConfigs {
    Table,
    AmmoType,
    AmmoTier,
    AtkRange,
}
