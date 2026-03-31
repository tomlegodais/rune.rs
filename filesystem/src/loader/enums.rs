use std::collections::HashMap;

use crate::{Cache, CacheResult, IndexId, definition::EnumType};

pub struct EnumLoader {
    types: HashMap<u32, EnumType>,
}

impl EnumLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut types = HashMap::new();
        let ref_table = cache.reference_table(IndexId::ENUMS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::ENUMS, archive_id)?;
            for (file_id, data) in files {
                let enum_id = archive_id.as_u32() * 256 + file_id.as_u32();

                match EnumType::decode(enum_id, &data) {
                    Ok(t) => {
                        types.insert(enum_id, t);
                    }
                    Err(e) => eprintln!("Warning: Failed to decode enum {}: {}", enum_id, e),
                }
            }
        }

        Ok(Self { types })
    }

    pub fn get(&self, id: u32) -> Option<&EnumType> {
        self.types.get(&id)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
