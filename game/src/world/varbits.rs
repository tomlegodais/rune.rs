use filesystem::definition::VarbitDefinition;
use filesystem::loader::VarbitLoader;
use filesystem::Cache;
use std::sync::OnceLock;

static INSTANCE: OnceLock<VarbitLoader> = OnceLock::new();

pub struct Varbits;

impl Varbits {
    pub fn init(cache: &Cache) {
        INSTANCE
            .get_or_init(|| VarbitLoader::load(cache).expect("failed to load varbit definitions"));
    }

    pub fn get(id: u32) -> Option<&'static VarbitDefinition> {
        INSTANCE.get().and_then(|l| l.get(id))
    }
}
