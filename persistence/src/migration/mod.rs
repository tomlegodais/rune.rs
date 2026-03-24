use sea_orm_migration::prelude::*;

mod m001_create_accounts;
mod m002_seed_accounts;
mod m003_create_players;
mod m004_add_running;
mod m005_add_run_energy;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_accounts::Migration),
            Box::new(m002_seed_accounts::Migration),
            Box::new(m003_create_players::Migration),
            Box::new(m004_add_running::Migration),
            Box::new(m005_add_run_energy::Migration),
        ]
    }
}
