use sea_orm_migration::prelude::*;

mod m001_create_accounts;
mod m002_seed_accounts;
mod m003_create_players;
mod m004_add_running;
mod m005_add_run_energy;
mod m006_create_inv;
mod m007_create_obj_configs;
mod m008_create_player_worn;
mod m009_add_obj_combat_stats;
mod m010_add_obj_weight;
mod m011_add_combat_settings;
mod m012_create_npc_configs;
mod m013_add_current_hp;
mod m014_add_obj_anim;

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
            Box::new(m006_create_inv::Migration),
            Box::new(m007_create_obj_configs::Migration),
            Box::new(m008_create_player_worn::Migration),
            Box::new(m009_add_obj_combat_stats::Migration),
            Box::new(m010_add_obj_weight::Migration),
            Box::new(m011_add_combat_settings::Migration),
            Box::new(m012_create_npc_configs::Migration),
            Box::new(m013_add_current_hp::Migration),
            Box::new(m014_add_obj_anim::Migration),
        ]
    }
}
