use std::collections::HashMap;

use crate::{ArchiveId, Cache, CacheResult, IndexId, definition::StructType};

pub struct StructLoader {
    definitions: HashMap<u32, StructType>,
}

impl StructLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut definitions = HashMap::new();
        let files = cache.read_all_files(IndexId::CONFIGS, ArchiveId::new(26))?;

        for (file_id, data) in files {
            let id = file_id.as_u32();
            match StructType::decode(id, &data) {
                Ok(def) => {
                    definitions.insert(id, def);
                }
                Err(e) => eprintln!("Warning: Failed to decode struct {}: {}", id, e),
            }
        }

        Ok(Self { definitions })
    }

    pub fn get(&self, id: u32) -> Option<&StructType> {
        self.definitions.get(&id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
