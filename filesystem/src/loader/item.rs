use crate::definition::{ItemDefinition, TransformKind};
use crate::{Cache, CacheResult, IndexId};
use std::collections::HashMap;

pub struct ItemLoader {
    definitions: HashMap<u32, ItemDefinition>,
}

impl ItemLoader {
    #[rustfmt::skip]
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut definitions = HashMap::new();
        let ref_table = cache.reference_table(IndexId::ITEMS)?;
        let decoded = ref_table.iter_archive_ids().flat_map(|archive_id| {
            cache
                .read_all_files(IndexId::ITEMS, archive_id)
                .into_iter()
                .flatten()
                .map(move |(file_id, data)| {
                    let item_id = archive_id.as_u32() * 256 + file_id.as_u32();
                    (item_id, ItemDefinition::decode(item_id, &data))
                })
        });

        for (item_id, result) in decoded {
            match result {
                Ok(def) => { definitions.insert(item_id, def); }
                Err(e) => eprintln!("Warning: Failed to decode item {}: {}", item_id, e),
            }
        }

        let transforms: Vec<(u32, TransformKind, u32)> = definitions
            .iter()
            .flat_map(|(&id, def)| {
                def.pending_transforms()
                    .map(move |(kind, source_id)| (id, kind, source_id))
            })
            .collect();

        for (id, kind, source_id) in transforms {
            let source = definitions[&source_id].clone();
            definitions
                .get_mut(&id)
                .unwrap()
                .apply_transform(kind, &source);
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
