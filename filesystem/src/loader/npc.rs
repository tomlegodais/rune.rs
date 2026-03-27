use crate::definition::NpcDefinition;
use crate::{Cache, CacheResult, IndexId};
use std::collections::HashMap;

pub struct NpcLoader {
    definitions: HashMap<u32, NpcDefinition>,
}

impl NpcLoader {
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut definitions = HashMap::new();
        let ref_table = cache.reference_table(IndexId::NPCS)?;

        for archive_id in ref_table.iter_archive_ids() {
            let files = cache.read_all_files(IndexId::NPCS, archive_id)?;
            for (file_id, data) in files {
                let npc_id = archive_id.as_u32() * 128 + file_id.as_u32();

                match NpcDefinition::decode(npc_id, &data) {
                    Ok(def) => {
                        definitions.insert(npc_id, def);
                    }
                    Err(e) => eprintln!("Warning: Failed to decode npc {}: {}", npc_id, e),
                }
            }
        }

        Ok(Self { definitions })
    }

    pub fn get(&self, id: u32) -> Option<&NpcDefinition> {
        self.definitions.get(&id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
