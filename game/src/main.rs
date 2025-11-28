use crate::config::AppConfig;
use crate::service::ServiceManager;
use ::config::{Config, Environment, File};
use filesystem::CacheBuilder;
use net::service::tcp::TcpService;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod config;
mod service;

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
    let tcp_service = TcpService::new(app_config.tcp, cache.clone())?;

    service_manager.spawn("TCP Service", |cancel, tx| async move {
        tcp_service.run_until(cancel.cancelled(), Some(tx)).await
    });

    service_manager
        .monitor()
        .on_ready(|| {
            info!("Ready to accept connections");
        })
        .await?;

    Ok(())
}
