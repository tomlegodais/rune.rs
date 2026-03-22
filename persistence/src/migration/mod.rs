use sea_orm_migration::prelude::*;

mod m001_create_accounts;
mod m002_seed_accounts;
mod m003_create_players;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_accounts::Migration),
            Box::new(m002_seed_accounts::Migration),
            Box::new(m003_create_players::Migration),
        ]
    }
}
