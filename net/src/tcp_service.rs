use crate::connection::Connection;
use crate::file_service::FileService;
use crate::tcp_config::TcpConfig;
use filesystem::Cache;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, oneshot};
use tracing::info;

pub struct TcpService {
    config: TcpConfig,
    cache: Arc<Cache>,
}

impl TcpService {
    pub fn new(config: TcpConfig, cache: Arc<Cache>) -> anyhow::Result<Self> {
        Ok(Self { config, cache })
    }

    async fn run(&self, on_ready: Option<oneshot::Sender<()>>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let semaphore = Arc::new(Semaphore::new(self.config.max_connections));
        let file_service = Arc::new(FileService::new(self.cache.clone())?);

        info!("Listening on {}", self.config.bind_addr);

        if let Some(tx) = on_ready {
            let _ = tx.send(());
        }

        loop {
            let permit = semaphore.clone().acquire_owned().await?;
            let (socket, _) = listener.accept().await?;
            let connection =
                Connection::new(socket, Arc::clone(&file_service), self.config.clone());

            tokio::spawn(async move {
                if let Err(_e) = connection.accept().await {
                    eprintln!("error accepting connection: {}", _e);
                }
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
