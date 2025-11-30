use crate::config::AppConfig;
use crate::service::{ServiceManager, WorldLoginService, WorldService};
use crate::world::World;
use filesystem::CacheBuilder;
use net::TcpService;
use std::sync::Arc;
use ::config::{Config, Environment, File};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod account;
mod config;
mod player;
mod service;
mod world;
mod message;

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
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let mut service_manager = ServiceManager::new();
    let cache = Arc::new(CacheBuilder::new("cache/").open()?);

    let world = Arc::new(Mutex::new(World::new()));
    let world_service = WorldService::new(world.clone());
    let login_service = Arc::new(WorldLoginService::new(app_config.game, world.clone()));
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
