use crate::provider::ProviderContext;
use filesystem::definition::NpcDefinition;
use filesystem::loader::NpcLoader;
use macros::data_provider;
use once_cell::sync::OnceCell;

static INSTANCE: OnceCell<NpcLoader> = OnceCell::new();

#[data_provider]
async fn load_npc_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE
        .get_or_try_init(|| NpcLoader::load(&ctx.cache))
        .map(drop)?)
}

pub fn get_npc_definition(id: u32) -> Option<&'static NpcDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
