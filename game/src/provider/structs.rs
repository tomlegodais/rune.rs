use filesystem::{definition::StructDefinition, loader::StructLoader};
use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<StructLoader> = OnceCell::new();

#[data_provider]
async fn load_struct_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE.get_or_try_init(|| StructLoader::load(&ctx.cache)).map(drop)?)
}

pub fn get_struct_definition(id: u32) -> Option<&'static StructDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
