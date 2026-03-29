use std::sync::Arc;

use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::{provider::ProviderContext, world::CollisionMap};

static INSTANCE: OnceCell<CollisionMap> = OnceCell::new();

#[data_provider(priority = 10)]
async fn load_collision(ctx: &ProviderContext) -> anyhow::Result<()> {
    INSTANCE
        .get_or_try_init(|| CollisionMap::new(Arc::clone(&ctx.cache)))
        .map(drop)
}

pub fn get_collision() -> &'static CollisionMap {
    INSTANCE.get().expect("collision map not initialized")
}
