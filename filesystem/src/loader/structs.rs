use std::collections::HashMap;

use crate::{ArchiveId, Cache, CacheResult, IndexId, definition::StructType};

pub struct StructLoader {
    types: HashMap<u32, StructType>,
}

impl StructLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut types = HashMap::new();
        let files = cache.read_all_files(IndexId::CONFIGS, ArchiveId::new(26))?;

        for (file_id, data) in files {
            let id = file_id.as_u32();
            match StructType::decode(id, &data) {
                Ok(t) => {
                    types.insert(id, t);
                }
                Err(e) => eprintln!("Warning: Failed to decode struct {}: {}", id, e),
            }
        }

        Ok(Self { types })
    }

    pub fn get(&self, id: u32) -> Option<&StructType> {
        self.types.get(&id)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
