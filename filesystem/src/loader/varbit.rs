use std::collections::HashMap;

use crate::{Cache, CacheResult, IndexId, definition::VarbitType};

pub struct VarbitLoader {
    definitions: HashMap<u32, VarbitType>,
}

impl VarbitLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut definitions = HashMap::new();
        let ref_table = cache.reference_table(IndexId::VARBITS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::VARBITS, archive_id)?;
            for (file_id, data) in files {
                let varbit_id = archive_id.as_u32() * 1024 + file_id.as_u32();

                match VarbitType::decode(varbit_id, &data) {
                    Ok(def) => {
                        definitions.insert(varbit_id, def);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to decode varbit {}: {}", varbit_id, e);
                    }
                }
            }
        }

        Ok(Self { definitions })
    }

    pub fn get(&self, id: u32) -> Option<&VarbitType> {
        self.definitions.get(&id)
    }
}
