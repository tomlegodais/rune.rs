use std::{sync::Arc, time::Duration};

use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

use crate::world::World;

const TICK_MS: Duration = Duration::from_millis(600);

#[derive(Clone)]
pub struct WorldService {
    world: Arc<World>,
}

impl WorldService {
    pub fn new(world: Arc<World>) -> Self {
        Self { world }
    }

    pub async fn run_until(&self, cancel: CancellationToken) {
        loop {
            let tick_start = Instant::now();

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = self.world.tick() => {}
            }

            let elapsed = tick_start.elapsed();
            if let Some(remaining) = TICK_MS.checked_sub(elapsed) {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = sleep(remaining) => {}
                }
            }
        }

        self.world.signal_logout_all().await;
    }
}
