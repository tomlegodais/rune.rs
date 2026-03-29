use crate::account::{AccountRepository, PgAccountRepository, PgAccountRepositoryParameters};
use crate::config::DatabaseConfig;
use crate::item::{ItemConfigRepository, PgItemConfigRepository, PgItemConfigRepositoryParameters};
use crate::migration::Migrator;
use crate::player::{PgPlayerRepository, PgPlayerRepositoryParameters, PlayerRepository};
use sea_orm::{ConnectOptions, Database as SeaDatabase};
use sea_orm_migration::MigratorTrait;
use shaku::module;

pub trait PersistenceModuleInterface:
    shaku::HasComponent<dyn AccountRepository>
    + shaku::HasComponent<dyn PlayerRepository>
    + shaku::HasComponent<dyn ItemConfigRepository>
{
}

module! {
    pub PersistenceModule: PersistenceModuleInterface {
        components = [PgAccountRepository, PgPlayerRepository, PgItemConfigRepository],
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
            db: db.clone(),
        })
        .with_component_parameters::<PgItemConfigRepository>(PgItemConfigRepositoryParameters {
            db,
        })
        .build();

    Ok(module)
}
