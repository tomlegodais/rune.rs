use filesystem::Cache;
use filesystem::definition::ItemDefinition;
use filesystem::loader::ItemLoader;
use macros::data_provider;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static INSTANCE: OnceCell<ItemLoader> = OnceCell::new();

#[data_provider]
pub fn load_item_definitions(cache: &Arc<Cache>) -> anyhow::Result<()> {
    Ok(INSTANCE
        .get_or_try_init(|| ItemLoader::load(cache))
        .map(drop)?)
}

pub fn get_item_definition(id: u32) -> Option<&'static ItemDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
