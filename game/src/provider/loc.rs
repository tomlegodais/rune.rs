use filesystem::Cache;
use filesystem::definition::LocDefinition;
use filesystem::loader::LocLoader;
use macros::data_provider;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static INSTANCE: OnceCell<LocLoader> = OnceCell::new();

#[data_provider]
pub fn load_item_definitions(cache: &Arc<Cache>) -> anyhow::Result<()> {
    Ok(INSTANCE
        .get_or_try_init(|| LocLoader::load(cache))
        .map(drop)?)
}

pub fn get_loc_definition(id: u32) -> Option<&'static LocDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
