use std::sync::Arc;

use filesystem::Cache;
use tokio::{
    net::TcpListener,
    sync::{Semaphore, oneshot},
};
use tokio_util::task::TaskTracker;

use crate::{LoginService, config::TcpConfig, service::cache::CacheService, session::Session};

pub struct TcpService {
    config: TcpConfig,
    cache: Arc<Cache>,
    login_service: Arc<dyn LoginService>,
}

impl TcpService {
    pub fn new(config: TcpConfig, cache: Arc<Cache>, login_service: Arc<dyn LoginService>) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            cache,
            login_service,
        })
    }

    async fn run(&self, on_ready: Option<oneshot::Sender<()>>, tracker: TaskTracker) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let semaphore = Arc::new(Semaphore::new(self.config.max_connections));
        let cache_service = Arc::new(CacheService::new(self.cache.clone())?);
        let login_service = Arc::clone(&self.login_service);

        tracing::info!(addr = %self.config.bind_addr, "Listening");

        if let Some(tx) = on_ready {
            let _ = tx.send(());
        }

        loop {
            let permit = semaphore.clone().acquire_owned().await?;
            let (socket, _) = listener.accept().await?;
            let session = Session::new(socket, Arc::clone(&cache_service), Arc::clone(&login_service));

            tracker.spawn(async move {
                if let Some(e) = session.run().await.err().filter(|err| !err.is_disconnect()) {
                    tracing::error!(error = %e, "Session Error");
                }

                drop(permit);
            });
        }
    }

    pub async fn run_until<F>(self, shutdown: F, on_ready: Option<oneshot::Sender<()>>) -> anyhow::Result<()>
    where
        F: Future<Output = ()>,
    {
        let tracker = TaskTracker::new();

        tokio::select! {
            result = self.run(on_ready, tracker.clone()) => result?,
            _ = shutdown => {}
        }

        tracker.close();

        if !tracker.is_empty() {
            tracing::info!(count = tracker.len(), "Waiting for active sessions to disconnect");
            tracker.wait().await;
        }

        Ok(())
    }
}
