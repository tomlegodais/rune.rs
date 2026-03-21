use crate::account::{PgAccountRepository, PgAccountRepositoryParameters};
use crate::config::DatabaseConfig;
use crate::migration::Migrator;
use crate::player::{PgPlayerRepository, PgPlayerRepositoryParameters};
use sea_orm::{ConnectOptions, Database as SeaDatabase};
use sea_orm_migration::MigratorTrait;
use shaku::module;

module! {
    pub PersistenceModule {
        components = [PgAccountRepository, PgPlayerRepository],
        providers = []
    }
}

pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<PersistenceModule> {
    let mut opts = ConnectOptions::new(&config.url);
    opts.max_connections(config.max_connections);

    let db = SeaDatabase::connect(opts).await?;
    Migrator::up(&db, None).await?;

    let module = PersistenceModule::builder()
        .with_component_parameters::<PgAccountRepository>(PgAccountRepositoryParameters {
            db: db.clone(),
        })
        .with_component_parameters::<PgPlayerRepository>(PgPlayerRepositoryParameters {
            db,
        })
        .build();

    Ok(module)
}