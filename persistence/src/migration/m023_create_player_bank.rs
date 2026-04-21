use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_BANK: &str = "[]";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlayerBank::Table)
                    .if_not_exists()
                    .col(big_integer(PlayerBank::PlayerId).primary_key().not_null())
                    .col(
                        ColumnDef::new(PlayerBank::Objs)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust(format!("'{DEFAULT_BANK}'::jsonb"))),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlayerBank::Table, PlayerBank::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO player_bank (player_id, objs) \
                 SELECT id, '{DEFAULT_BANK}'::jsonb FROM players \
                 ON CONFLICT (player_id) DO NOTHING"
            ))
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlayerBank::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PlayerBank {
    Table,
    PlayerId,
    Objs,
}

#[derive(DeriveIden)]
enum Players {
    Table,
    Id,
}
