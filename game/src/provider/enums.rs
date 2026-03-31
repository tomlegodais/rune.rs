use filesystem::{config::EnumType, loader::EnumLoader};
use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<EnumLoader> = OnceCell::new();

#[data_provider]
async fn load_enum_types(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE.get_or_try_init(|| EnumLoader::load(&ctx.cache)).map(drop)?)
}

pub fn get_enum_type(id: u32) -> Option<&'static EnumType> {
    INSTANCE.get().and_then(|l| l.get(id))
}
