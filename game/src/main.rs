use crate::config::AppConfig;
use crate::service::{
    GameLoginService, ServiceManager, WorldLoginService, WorldLoginServiceParameters, WorldService,
};
use crate::world::World;
use ::config::{Config, Environment, File};
use filesystem::CacheBuilder;
use net::TcpService;
use persistence::PersistenceModuleInterface;
use shaku::{HasComponent, module};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod command;
mod config;
mod entity;
mod handler;
mod npc;
mod player;
mod provider;
mod service;
mod world;

module! {
    GameModule {
        components = [WorldLoginService],
        providers = [],

        use dyn PersistenceModuleInterface {
            components = [
                dyn persistence::account::AccountRepository,
                dyn persistence::player::PlayerRepository,
            ],
            providers = []
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::builder()
        .add_source(File::with_name("Config").required(false))
        .add_source(File::with_name("Config.local").required(false))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;

    let app_config: AppConfig = config.try_deserialize()?;
    let filter = EnvFilter::builder()
        .with_default_directive(app_config.log.level.parse()?)
        .from_env_lossy()
        .add_directive("sqlx=warn".parse()?)
        .add_directive("sea_orm_migration=warn".parse()?);

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let mut service_manager = ServiceManager::new();
    let cache = Arc::new(CacheBuilder::new("cache/").open()?);
    provider::load_all(&cache)?;

    let world = Arc::new(World::new());
    let persistence = Arc::new(persistence::connect(&app_config.database).await?);
    let game = GameModule::builder(persistence)
        .with_component_parameters::<WorldLoginService>(WorldLoginServiceParameters {
            config: app_config.game,
            world: world.clone(),
        })
        .build();

    let login_service: Arc<dyn GameLoginService> = game.resolve();
    let world_service = WorldService::new(world.clone());
    let tcp_service = TcpService::new(app_config.tcp, cache.clone(), login_service)?;

    service_manager.spawn("TCP Service", |cancel, tx| async move {
        tcp_service.run_until(cancel.cancelled(), Some(tx)).await
    });

    service_manager.spawn("World Service", move |cancel, tx| {
        let world_service = world_service.clone();
        async move {
            let _ = tx.send(());
            world_service.run_until(cancel).await;
            Ok(())
        }
    });

    service_manager
        .monitor()
        .on_ready(|| {
            info!("Ready to accept connections");
        })
        .await?;

    Ok(())
}
