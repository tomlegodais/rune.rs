use filesystem::{definition::StructType, loader::StructLoader};
use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<StructLoader> = OnceCell::new();

#[data_provider]
async fn load_struct_types(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE.get_or_try_init(|| StructLoader::load(&ctx.cache)).map(drop)?)
}

pub fn get_struct_type(id: u32) -> Option<&'static StructType> {
    INSTANCE.get().and_then(|l| l.get(id))
}
