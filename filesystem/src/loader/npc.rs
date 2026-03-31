use std::collections::HashMap;

use crate::{Cache, CacheResult, IndexId, config::NpcType};

pub struct NpcLoader {
    types: HashMap<u32, NpcType>,
}

impl NpcLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut types = HashMap::new();
        let ref_table = cache.reference_table(IndexId::NPCS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::NPCS, archive_id)?;
            for (file_id, data) in files {
                let npc_id = archive_id.as_u32() * 128 + file_id.as_u32();

                match NpcType::decode(npc_id, &data) {
                    Ok(t) => {
                        types.insert(npc_id, t);
                    }
                    Err(e) => eprintln!("Warning: Failed to decode npc {}: {}", npc_id, e),
                }
            }
        }

        Ok(Self { types })
    }

    pub fn get(&self, id: u32) -> Option<&NpcType> {
        self.types.get(&id)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
