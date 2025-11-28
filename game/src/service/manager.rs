use crate::service::monitor::ServiceMonitor;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

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
                    info!("Service '{}' stopped gracefully", name);
                    Ok(())
                }

                Err(e) => {
                    error!("Service '{}' failed: {:?}", name, e);
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

    pub (super) async fn await_ready(&mut self) -> anyhow::Result<()> {
        info!("Waiting for services to initialize");

        for (name, rx) in self.start_checks.drain(..) {
            match rx.await {
                Ok(_) => info!("Service '{}' is ready", name),
                Err(_) => {
                    let msg = format!("Service '{}' failed to start (channel closed)!", name);
                    error!("{}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }
        }

        Ok(())
    }

    pub (super) async fn join(&mut self) {
        while let Some(_) = self.set.join_next().await {}
    }
}
