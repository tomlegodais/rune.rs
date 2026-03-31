use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_STATS: &str = r#"[
    {"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},{"level":10,"xp":1154},
    {"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},
    {"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},
    {"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},
    {"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},
    {"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0},{"level":1,"xp":0}
]"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Players::Table)
                    .if_not_exists()
                    .col(pk_auto(Players::Id).big_integer())
                    .col(big_integer(Players::AccountId).unique_key().not_null())
                    .col(integer(Players::X).not_null().default(3093))
                    .col(integer(Players::Y).not_null().default(3493))
                    .col(integer(Players::Plane).not_null().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Players::Table, Players::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PlayerAppearance::Table)
                    .if_not_exists()
                    .col(big_integer(PlayerAppearance::PlayerId).primary_key().not_null())
                    .col(boolean(PlayerAppearance::Male).not_null().default(true))
                    .col(
                        ColumnDef::new(PlayerAppearance::Look)
                            .array(ColumnType::SmallInteger)
                            .not_null()
                            .default(Expr::cust("ARRAY[8,14,18,26,34,38,42]::smallint[]")),
                    )
                    .col(
                        ColumnDef::new(PlayerAppearance::Colors)
                            .array(ColumnType::SmallInteger)
                            .not_null()
                            .default(Expr::cust("ARRAY[5,15,15,0,0]::smallint[]")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlayerAppearance::Table, PlayerAppearance::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PlayerStats::Table)
                    .if_not_exists()
                    .col(big_integer(PlayerStats::PlayerId).primary_key().not_null())
                    .col(
                        ColumnDef::new(PlayerStats::Stats)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust(format!("'{DEFAULT_STATS}'::jsonb"))),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PlayerStats::Table, PlayerStats::PlayerId)
                            .to(Players::Table, Players::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlayerStats::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PlayerAppearance::Table).to_owned())
            .await?;
        manager.drop_table(Table::drop().table(Players::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Players {
    Table,
    Id,
    AccountId,
    X,
    Y,
    Plane,
}

#[derive(DeriveIden)]
enum PlayerAppearance {
    Table,
    PlayerId,
    Male,
    Look,
    Colors,
}

#[derive(DeriveIden)]
enum PlayerStats {
    Table,
    PlayerId,
    Stats,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}
