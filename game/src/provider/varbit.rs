use filesystem::{definition::VarbitDefinition, loader::VarbitLoader};
use macros::data_provider;
use once_cell::sync::OnceCell;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<VarbitLoader> = OnceCell::new();

#[data_provider]
async fn load_varbit_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE.get_or_try_init(|| VarbitLoader::load(&ctx.cache)).map(drop)?)
}

pub fn get_varbit_definition(id: u32) -> Option<&'static VarbitDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
