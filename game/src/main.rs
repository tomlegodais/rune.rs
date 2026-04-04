use std::sync::Arc;

use ::config::{Config, Environment, File};
use filesystem::CacheBuilder;
use net::TcpService;
use persistence::PersistenceModuleInterface;
use shaku::{HasComponent, module};
use tracing_subscriber::EnvFilter;

use crate::{
    config::AppConfig,
    provider::ProviderContext,
    service::{GameLoginService, ServiceManager, WorldLoginService, WorldLoginServiceParameters, WorldService},
    world::World,
};

mod command;
mod config;
mod content;
mod entity;
mod fmt;
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

    let _ = enable_ansi_support::enable_ansi_support();
    tracing_subscriber::fmt()
        .event_format(fmt::Formatter)
        .with_env_filter(filter)
        .init();

    let mut service_manager = ServiceManager::new();
    let cache = Arc::new(CacheBuilder::new("cache/").open()?);
    let persistence = Arc::new(persistence::connect(&app_config.database).await?);
    provider::load_all(&ProviderContext {
        cache: cache.clone(),
        persistence: persistence.clone(),
    })
    .await?;

    let world = Arc::new(World::default());
    world.init();

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
            tracing::info!("Ready to accept connections");
        })
        .await?;

    Ok(())
}
