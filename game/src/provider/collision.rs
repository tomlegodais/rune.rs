use crate::world::CollisionMap;
use filesystem::Cache;
use macros::data_provider;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static INSTANCE: OnceCell<CollisionMap> = OnceCell::new();

#[data_provider(priority = 10)]
fn load_collision(cache: &Arc<Cache>) -> anyhow::Result<()> {
    INSTANCE
        .get_or_try_init(|| CollisionMap::new(Arc::clone(cache)))
        .map(drop)
}

pub fn get_collision() -> &'static CollisionMap {
    INSTANCE.get().expect("collision map not initialized")
}
