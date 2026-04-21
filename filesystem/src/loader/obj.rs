use std::collections::HashMap;

use crate::{
    Cache, CacheResult, IndexId,
    config::{ObjType, TransformKind},
};

pub struct ObjLoader {
    types: HashMap<u32, ObjType>,
}

impl ObjLoader {
    #[rustfmt::skip]
    pub fn load(cache: &Cache) -> CacheResult<Self> {
        let mut types = HashMap::new();
        let ref_table = cache.reference_table(IndexId::OBJS)?;
        let decoded = ref_table.iter_archive_ids().flat_map(|archive_id| {
            cache
                .read_all_files(IndexId::OBJS, archive_id)
                .into_iter()
                .flatten()
                .map(move |(file_id, data)| {
                    let obj_id = archive_id.as_u32() * 256 + file_id.as_u32();
                    (obj_id, ObjType::decode(obj_id, &data))
                })
        });

        for (obj_id, result) in decoded {
            match result {
                Ok(t) => { types.insert(obj_id, t); }
                Err(e) => eprintln!("Warning: Failed to decode obj {}: {}", obj_id, e),
            }
        }

        let transforms: Vec<(u32, TransformKind, u32)> = types
            .iter()
            .flat_map(|(&id, t)| {
                t.pending_transforms()
                    .map(move |(kind, source_id)| (id, kind, source_id))
            })
            .collect();

        for (id, kind, source_id) in transforms {
            let source = types[&source_id].clone();
            types
                .get_mut(&id)
                .unwrap()
                .apply_transform(kind, &source);
        }

        Ok(Self { types })
    }

    pub fn get(&self, id: u32) -> Option<&ObjType> {
        self.types.get(&id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut ObjType> {
        self.types.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
