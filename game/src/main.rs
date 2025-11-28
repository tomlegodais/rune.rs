use crate::service::ServiceManager;
use filesystem::CacheBuilder;
use net::tcp_config::TcpConfig;
use net::tcp_service::TcpService;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut service_manager = ServiceManager::new();
    let cache = Arc::new(CacheBuilder::new("cache/").open()?);
    let tcp_service = TcpService::new(TcpConfig::default(), cache.clone())?;

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
