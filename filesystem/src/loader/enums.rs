use std::collections::HashMap;

use crate::{Cache, CacheResult, IndexId, definition::EnumType};

pub struct EnumLoader {
    definitions: HashMap<u32, EnumType>,
}

impl EnumLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut definitions = HashMap::new();
        let ref_table = cache.reference_table(IndexId::ENUMS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::ENUMS, archive_id)?;
            for (file_id, data) in files {
                let enum_id = archive_id.as_u32() * 256 + file_id.as_u32();

                match EnumType::decode(enum_id, &data) {
                    Ok(def) => {
                        definitions.insert(enum_id, def);
                    }
                    Err(e) => eprintln!("Warning: Failed to decode enum {}: {}", enum_id, e),
                }
            }
        }

        Ok(Self { definitions })
    }

    pub fn get(&self, id: u32) -> Option<&EnumType> {
        self.definitions.get(&id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
