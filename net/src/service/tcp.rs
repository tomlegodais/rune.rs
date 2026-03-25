use crate::LoginService;
use crate::config::TcpConfig;
use crate::service::cache::CacheService;
use crate::session::Session;
use filesystem::Cache;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, oneshot};
use tracing::{error, info};

pub struct TcpService {
    config: TcpConfig,
    cache: Arc<Cache>,
    login_service: Arc<dyn LoginService>,
}

impl TcpService {
    pub fn new(
        config: TcpConfig,
        cache: Arc<Cache>,
        login_service: Arc<dyn LoginService>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            cache,
            login_service,
        })
    }

    async fn run(&self, on_ready: Option<oneshot::Sender<()>>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let semaphore = Arc::new(Semaphore::new(self.config.max_connections));
        let cache_service = Arc::new(CacheService::new(self.cache.clone())?);
        let login_service = Arc::clone(&self.login_service);

        info!("Listening on {}", self.config.bind_addr);

        if let Some(tx) = on_ready {
            let _ = tx.send(());
        }

        loop {
            let permit = semaphore.clone().acquire_owned().await?;
            let (socket, _) = listener.accept().await?;
            let session = Session::new(
                socket,
                Arc::clone(&cache_service),
                Arc::clone(&login_service),
            );

            tokio::spawn(async move {
                session
                    .run()
                    .await
                    .err()
                    .filter(|err| !err.is_disconnect())
                    .map(|e| error!("{e}"));

                drop(permit);
            });
        }
    }

    pub async fn run_until<F>(
        self,
        shutdown: F,
        on_ready: Option<oneshot::Sender<()>>,
    ) -> anyhow::Result<()>
    where
        F: Future<Output = ()>,
    {
        tokio::select! {
            result = self.run(on_ready) => result,
            _ = shutdown => Ok(())
        }
    }
}
