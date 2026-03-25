use crate::service::ServiceManager;
use std::pin::Pin;

pub struct ServiceMonitor<F> {
    pub manager: ServiceManager,
    pub on_ready: F,
}

impl<F> ServiceMonitor<F>
where
    F: FnOnce() + Send,
{
    pub fn on_ready<NewF>(self, callback: NewF) -> ServiceMonitor<NewF>
    where
        NewF: FnOnce() + Send,
    {
        ServiceMonitor {
            manager: self.manager,
            on_ready: callback,
        }
    }
}

impl<F> IntoFuture for ServiceMonitor<F>
where
    F: FnOnce() + Send + 'static,
{
    type Output = anyhow::Result<()>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        let mut manager = self.manager;
        let on_ready = self.on_ready;

        Box::pin(async move {
            manager.await_ready().await?;
            on_ready();
            manager.join().await;

            Ok(())
        })
    }
}
