use filesystem::{LocLoader, LocType};
use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<LocLoader> = OnceCell::new();

#[data_provider]
async fn load_loc_types(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE.get_or_try_init(|| LocLoader::load(&ctx.cache)).map(drop)?)
}

pub fn get_loc_type(id: u32) -> Option<&'static LocType> {
    INSTANCE.get().and_then(|l| l.get(id))
}
