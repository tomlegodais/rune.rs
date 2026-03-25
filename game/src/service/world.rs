use crate::world::World;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

const TICK_MS: Duration = Duration::from_millis(600);

#[derive(Clone)]
pub struct WorldService {
    world: Arc<Mutex<World>>,
}

impl WorldService {
    pub fn new(world: Arc<Mutex<World>>) -> Self {
        Self { world }
    }

    pub async fn run_until(&self, cancel: CancellationToken) {
        loop {
            let tick_start = Instant::now();

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = async {
                    {
                        let mut world = self.world.lock().await;
                        world.tick().await;
                    }
                } => {}
            }

            let elapsed = tick_start.elapsed();
            if let Some(remaining) = TICK_MS.checked_sub(elapsed) {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = sleep(remaining) => {}
                }
            }
        }
    }
}
