use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_WORN: &str = "[null,null,null,null,null,null,null,null,null,null,null,null,null,null]";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlayerWorn::Table)
                    .if_not_exists()
                    .col(big_integer(PlayerWorn::PlayerId).primary_key().not_null())
                    .col(
                        ColumnDef::new(PlayerWorn::Items)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust(format!("'{DEFAULT_WORN}'::jsonb"))),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlayerWorn::Table, PlayerWorn::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO player_worn (player_id, items) \
                 SELECT id, '{DEFAULT_WORN}'::jsonb FROM players \
                 ON CONFLICT (player_id) DO NOTHING"
            ))
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlayerWorn::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PlayerWorn {
    Table,
    PlayerId,
    Items,
}

#[derive(DeriveIden)]
enum Players {
    Table,
    Id,
}
