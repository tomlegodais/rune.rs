use crate::config::Js5Config;
use crate::connection::Js5Connection;
use crate::service::Js5Service;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

pub struct Js5Server {
    config: Js5Config,
    service: Arc<Js5Service>,
}

impl Js5Server {
    pub fn new(config: Js5Config, service: Js5Service) -> Self {
        Self {
            config,
            service: Arc::new(service),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let semaphore = Arc::new(Semaphore::new(self.config.max_connections));

        loop {
            let permit = semaphore.clone().acquire_owned().await?;
            let (socket, _) = listener.accept().await?;
            let connection =
                Js5Connection::new(socket, Arc::clone(&self.service), self.config.clone());

            tokio::spawn(async move {
                if let Err(_e) = connection.accept().await {
                    eprintln!("error accepting connection: {}", _e);
                }
                drop(permit);
            });
        }
    }

    pub async fn run_until<F>(self, shutdown: F) -> anyhow::Result<()>
    where
        F: Future<Output = ()>,
    {
        tokio::select! {
            result = self.run() => result,
            _ = shutdown => {
                println!("Js5 server shutting down...");
                Ok(())
            }
        }
    }
}
