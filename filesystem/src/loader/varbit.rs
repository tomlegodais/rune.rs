use std::collections::HashMap;

use crate::{Cache, CacheResult, IndexId, config::VarbitType};

pub struct VarbitLoader {
    types: HashMap<u32, VarbitType>,
}

impl VarbitLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut types = HashMap::new();
        let ref_table = cache.reference_table(IndexId::VARBITS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::VARBITS, archive_id)?;
            for (file_id, data) in files {
                let varbit_id = archive_id.as_u32() * 1024 + file_id.as_u32();

                match VarbitType::decode(varbit_id, &data) {
                    Ok(t) => {
                        types.insert(varbit_id, t);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to decode varbit {}: {}", varbit_id, e);
                    }
                }
            }
        }

        Ok(Self { types })
    }

    pub fn get(&self, id: u32) -> Option<&VarbitType> {
        self.types.get(&id)
    }
}
