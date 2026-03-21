use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(pk_auto(Accounts::Id).big_integer())
                    .col(ColumnDef::new(Accounts::Username).custom(Alias::new("citext")).unique_key().not_null())
                    .col(string(Accounts::PasswordHash).not_null())
                    .col(small_integer(Accounts::Rights).not_null().default(0))
                    .col(boolean(Accounts::Disabled).not_null().default(false))
                    .col(timestamp_with_time_zone(Accounts::CreatedAt).not_null().default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone_null(Accounts::LastLogin))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Accounts::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Username,
    PasswordHash,
    Rights,
    Disabled,
    CreatedAt,
    LastLogin,
}