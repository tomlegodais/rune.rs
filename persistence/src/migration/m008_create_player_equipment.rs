use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_EQUIPMENT: &str =
    "[null,null,null,null,null,null,null,null,null,null,null,null,null,null]";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlayerEquipment::Table)
                    .if_not_exists()
                    .col(
                        big_integer(PlayerEquipment::PlayerId)
                            .primary_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlayerEquipment::Items)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust(format!("'{DEFAULT_EQUIPMENT}'::jsonb"))),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlayerEquipment::Table, PlayerEquipment::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO player_equipment (player_id, items) \
                 SELECT id, '{DEFAULT_EQUIPMENT}'::jsonb FROM players \
                 ON CONFLICT (player_id) DO NOTHING"
            ))
            .await
            .map(|_| ())?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlayerEquipment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PlayerEquipment {
    Table,
    PlayerId,
    Items,
}

#[derive(DeriveIden)]
enum Players {
    Table,
    Id,
}
