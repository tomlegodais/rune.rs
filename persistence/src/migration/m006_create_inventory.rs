use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_INVENTORY: &str = r#"[
    {"item_id":1205,"amount":1},
    {"item_id":1171,"amount":1},
    {"item_id":2309,"amount":1},
    null,null,null,null,
    null,null,null,null,
    null,null,null,null,
    null,null,null,null,
    null,null,null,null,
    null,null,null,null,
    null,null
]"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlayerInventory::Table)
                    .if_not_exists()
                    .col(
                        big_integer(PlayerInventory::PlayerId)
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlayerInventory::Items)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust(format!("'{DEFAULT_INVENTORY}'::jsonb"))),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlayerInventory::Table, PlayerInventory::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO player_inventory (player_id, items) \
                 SELECT id, '{DEFAULT_INVENTORY}'::jsonb FROM players \
                 ON CONFLICT (player_id) DO NOTHING"
            ))
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlayerInventory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PlayerInventory {
    Table,
    PlayerId,
    Items,
}

#[derive(DeriveIden)]
enum Players {
    Table,
    Id,
}
