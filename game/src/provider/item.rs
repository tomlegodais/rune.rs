use crate::provider::ProviderContext;
use filesystem::definition::ItemDefinition;
use filesystem::loader::ItemLoader;
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::item::ItemConfigRepository;
use shaku::HasComponent;
use std::sync::Arc;

static INSTANCE: OnceCell<ItemLoader> = OnceCell::new();

#[data_provider]
async fn load_item_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    let mut loader = ItemLoader::load(&ctx.cache)?;

    let repo: Arc<dyn ItemConfigRepository> = ctx.persistence.resolve();
    let configs = repo.find_all().await?;

    for config in configs {
        if let Some(def) = loader.get_mut(config.item_id) {
            def.equipment_slot = config.equipment_slot;
            def.two_handed = config.two_handed;
        }
    }

    INSTANCE
        .set(loader)
        .map_err(|_| anyhow::anyhow!("item definitions already loaded"))
}

pub fn get_item_definition(id: u32) -> Option<&'static ItemDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
