use std::collections::HashMap;

use crate::{Cache, CacheResult, IndexId, definition::LocType};

pub struct LocLoader {
    types: HashMap<u32, LocType>,
}

impl LocLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut types = HashMap::new();
        let ref_table = cache.reference_table(IndexId::LOCS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::LOCS, archive_id)?;
            for (file_id, data) in files {
                let loc_id = archive_id.as_u32() * 256 + file_id.as_u32();

                match LocType::decode(loc_id, &data) {
                    Ok(t) => {
                        types.insert(loc_id, t);
                    }
                    Err(e) => eprintln!("Warning: Failed to decode loc {}: {}", loc_id, e),
                }
            }
        }

        Ok(Self { types })
    }

    pub fn get(&self, id: u32) -> Option<&LocType> {
        self.types.get(&id)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
