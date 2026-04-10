use sea_orm::{ConnectOptions, Database as SeaDatabase};
use sea_orm_migration::MigratorTrait;
use shaku::module;

use crate::{
    account::{AccountRepository, PgAccountRepository, PgAccountRepositoryParameters},
    config::DatabaseConfig,
    migration::Migrator,
    npc::{NpcConfigRepository, PgNpcConfigRepository, PgNpcConfigRepositoryParameters},
    obj::{ObjConfigRepository, PgObjConfigRepository, PgObjConfigRepositoryParameters},
    player::{PgPlayerRepository, PgPlayerRepositoryParameters, PlayerRepository},
};

pub trait PersistenceModuleInterface:
    shaku::HasComponent<dyn AccountRepository>
    + shaku::HasComponent<dyn PlayerRepository>
    + shaku::HasComponent<dyn ObjConfigRepository>
    + shaku::HasComponent<dyn NpcConfigRepository>
{
}

module! {
    pub PersistenceModule: PersistenceModuleInterface {
        components = [PgAccountRepository, PgPlayerRepository, PgObjConfigRepository, PgNpcConfigRepository],
        providers = []
    }
}

pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<PersistenceModule> {
    let mut opts = ConnectOptions::new(&config.url);
    opts.max_connections(config.max_connections);

    let db = SeaDatabase::connect(opts).await?;
    Migrator::up(&db, None).await?;

    let module = PersistenceModule::builder()
        .with_component_parameters::<PgAccountRepository>(PgAccountRepositoryParameters { db: db.clone() })
        .with_component_parameters::<PgPlayerRepository>(PgPlayerRepositoryParameters { db: db.clone() })
        .with_component_parameters::<PgObjConfigRepository>(PgObjConfigRepositoryParameters { db: db.clone() })
        .with_component_parameters::<PgNpcConfigRepository>(PgNpcConfigRepositoryParameters { db })
        .build();

    Ok(module)
}
