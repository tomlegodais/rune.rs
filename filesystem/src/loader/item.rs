use crate::definition::ItemDefinition;
use crate::{Cache, CacheResult, IndexId};
use std::collections::HashMap;

pub struct ItemLoader {
    definitions: HashMap<u32, ItemDefinition>,
}

impl ItemLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut definitions = HashMap::new();
        let ref_table = cache.reference_table(IndexId::ITEMS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::ITEMS, archive_id)?;
            for (file_id, data) in files {
                let item_id = archive_id.as_u32() * 256 + file_id.as_u32();

                match ItemDefinition::decode(item_id, &data) {
                    Ok(def) => {
                        definitions.insert(item_id, def);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to decode item {}: {}", item_id, e);
                    }
                }
            }
        }

        Ok(Self { definitions })
    }

    pub fn get(&self, id: u32) -> Option<&ItemDefinition> {
        self.definitions.get(&id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }
}
