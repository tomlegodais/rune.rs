use filesystem::Cache;
use filesystem::definition::VarbitDefinition;
use filesystem::loader::VarbitLoader;
use macros::data_provider;
use once_cell::sync::OnceCell;
use std::sync::Arc;

static INSTANCE: OnceCell<VarbitLoader> = OnceCell::new();

#[data_provider]
pub fn load_varbit_definitions(cache: &Arc<Cache>) -> anyhow::Result<()> {
    Ok(INSTANCE
        .get_or_try_init(|| VarbitLoader::load(cache))
        .map(drop)?)
}

pub fn get_varbit_definition(id: u32) -> Option<&'static VarbitDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
