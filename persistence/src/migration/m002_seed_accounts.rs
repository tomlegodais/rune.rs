use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO accounts (username, password_hash, rights) \
                 VALUES ('tom', '$argon2id$v=19$m=19456,t=2,p=1$U2faud9cRjW7G5jyddcyZg$miqHk358nJLT9rFALyNoL9bcpdHyE1cG5ZAjA5KXVOo', 2)",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DELETE FROM accounts WHERE username = 'tom'")
            .await?;

        Ok(())
    }
}
