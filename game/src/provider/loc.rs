use crate::provider::ProviderContext;
use filesystem::definition::LocDefinition;
use filesystem::loader::LocLoader;
use macros::data_provider;
use once_cell::sync::OnceCell;

static INSTANCE: OnceCell<LocLoader> = OnceCell::new();

#[data_provider]
async fn load_loc_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE
        .get_or_try_init(|| LocLoader::load(&ctx.cache))
        .map(drop)?)
}

pub fn get_loc_definition(id: u32) -> Option<&'static LocDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
