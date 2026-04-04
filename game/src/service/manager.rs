use tokio::{sync::oneshot, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::service::monitor::ServiceMonitor;

pub struct ServiceManager {
    token: CancellationToken,
    set: JoinSet<anyhow::Result<()>>,
    start_checks: Vec<(&'static str, oneshot::Receiver<()>)>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let cancellation_token = CancellationToken::new();
        let signal_token = cancellation_token.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            signal_token.cancel();
        });

        Self {
            token: cancellation_token,
            set: JoinSet::new(),
            start_checks: Vec::new(),
        }
    }

    pub fn spawn<F, Fut>(&mut self, name: &'static str, factory: F)
    where
        F: FnOnce(CancellationToken, oneshot::Sender<()>) -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let token = self.token.clone();
        let (tx, rx) = oneshot::channel();

        self.start_checks.push((name, rx));
        self.set.spawn(async move {
            match factory(token, tx).await {
                Ok(_) => {
                    tracing::info!(name, "Service stopped gracefully");
                    Ok(())
                }

                Err(e) => {
                    tracing::error!(name, error = ?e, "Service failed");
                    Err(e)
                }
            }
        });
    }

    pub fn monitor(self) -> ServiceMonitor<impl FnOnce() + Send> {
        ServiceMonitor {
            manager: self,
            on_ready: || {},
        }
    }

    pub(super) async fn await_ready(&mut self) -> anyhow::Result<()> {
        tracing::info!("Waiting for services to initialize");

        for (name, rx) in self.start_checks.drain(..) {
            match rx.await {
                Ok(_) => tracing::info!(name, "Service is ready"),
                Err(_) => {
                    tracing::error!(name, "Service failed to start (channel closed)");
                    return Err(anyhow::anyhow!("Service '{name}' failed to start (channel closed)!"));
                }
            }
        }

        Ok(())
    }

    pub(super) async fn join(&mut self) {
        while (self.set.join_next().await).is_some() {}
    }
}
