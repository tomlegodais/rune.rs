use crate::archive::unpack_archive_owned;
use crate::codec::decode_container;
use crate::error::{CacheError, CacheResult};
use crate::id::{ArchiveId, FileId, IndexId, REFERENCE_INDEX};
use crate::reference::{ReferenceTable, name_hash};
use crate::store::{DataStore, IndexStore};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

pub struct Cache {
    data_store: DataStore,
    index_stores: HashMap<IndexId, IndexStore>,
    reference_tables: RwLock<HashMap<IndexId, ReferenceTable>>,
}

pub struct CacheBuilder {
    path: PathBuf,
    preload_references: bool,
}

impl Cache {
    pub fn open(path: impl AsRef<Path>) -> CacheResult<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() || !path.is_dir() {
            return Err(CacheError::DirectoryNotFound(path));
        }

        let data_path = path.join("main_file_cache.dat2");
        let data_store = DataStore::open(&data_path)?;
        let mut index_stores = HashMap::new();
        for i in 0..=255u8 {
            let index_id = IndexId::new(i);
            let index_path = path.join(format!("main_file_cache.idx{}", i));
            if index_path.exists() {
                match IndexStore::open(&index_path, index_id) {
                    Ok(store) => {
                        index_stores.insert(index_id, store);
                    }
                    Err(_) => continue,
                }
            }
        }

        Ok(Self {
            data_store,
            index_stores,
            reference_tables: RwLock::new(HashMap::new()),
        })
    }

    pub fn indices(&self) -> impl Iterator<Item = IndexId> + '_ {
        self.index_stores
            .keys()
            .copied()
            .filter(|id| !id.is_reference())
    }

    pub fn has_index(&self, index: IndexId) -> bool {
        self.index_stores.contains_key(&index)
    }

    pub fn archive_count(&self, index: IndexId) -> CacheResult<u32> {
        self.index_stores
            .get(&index)
            .map(|store| store.archive_count())
            .ok_or(CacheError::IndexNotExists(index))
    }

    pub fn read_archive_raw(&self, index: IndexId, archive: ArchiveId) -> CacheResult<Vec<u8>> {
        let index_store = self
            .index_stores
            .get(&index)
            .ok_or(CacheError::IndexNotExists(index))?;

        let (size, sector) = index_store
            .get_entry(archive)
            .ok_or(CacheError::ArchiveNotFound { index, archive })?;

        self.data_store.read_archive(index, archive, sector, size)
    }

    pub fn read_archive(&self, index: IndexId, archive: ArchiveId) -> CacheResult<Vec<u8>> {
        let raw = self.read_archive_raw(index, archive)?;
        decode_container(&raw)
    }

    pub fn reference_table(&self, index: IndexId) -> CacheResult<ReferenceTable> {
        {
            let tables = self.reference_tables.read().unwrap();
            if let Some(table) = tables.get(&index) {
                return Ok(table.clone());
            }
        }

        let raw = self.read_archive(REFERENCE_INDEX, ArchiveId::new(index.as_u8() as u32))?;
        let table = ReferenceTable::parse(&raw)?;
        {
            let mut tables = self.reference_tables.write().unwrap();
            tables.insert(index, table.clone());
        }

        Ok(table)
    }

    pub fn read_file(
        &self,
        index: IndexId,
        archive: ArchiveId,
        file: FileId,
    ) -> CacheResult<Vec<u8>> {
        let ref_table = self.reference_table(index)?;
        let archive_entry = ref_table
            .archive(archive)
            .ok_or(CacheError::ArchiveNotFound { index, archive })?;

        if !archive_entry.files.contains_key(&file) {
            return Err(CacheError::FileNotFound { archive, file });
        }

        let data = self.read_archive(index, archive)?;
        let file_ids: Vec<FileId> = archive_entry.files.keys().copied().collect();
        let files = unpack_archive_owned(&data, &file_ids)?;
        files
            .into_iter()
            .find(|(id, _)| *id == file)
            .map(|(_, data)| data)
            .ok_or(CacheError::FileNotFound { archive, file })
    }

    pub fn read_all_files(
        &self,
        index: IndexId,
        archive: ArchiveId,
    ) -> CacheResult<HashMap<FileId, Vec<u8>>> {
        let ref_table = self.reference_table(index)?;
        let archive_entry = ref_table
            .archive(archive)
            .ok_or(CacheError::ArchiveNotFound { index, archive })?;

        let data = self.read_archive(index, archive)?;
        let file_ids: Vec<FileId> = archive_entry.files.keys().copied().collect();

        let unpacked = unpack_archive_owned(&data, &file_ids)?;
        Ok(unpacked.into_iter().collect())
    }

    pub fn find_archive(&self, index: IndexId, name: &str) -> CacheResult<Option<ArchiveId>> {
        let ref_table = self.reference_table(index)?;
        let hash = name_hash(name);
        Ok(ref_table.find_by_name(hash).map(|(id, _)| id))
    }

    pub fn read_named_file(
        &self,
        index: IndexId,
        archive_name: &str,
        file_name: &str,
    ) -> CacheResult<Vec<u8>> {
        let ref_table = self.reference_table(index)?;

        let archive_hash = name_hash(archive_name);
        let (archive_id, archive_entry) =
            ref_table
                .find_by_name(archive_hash)
                .ok_or_else(|| CacheError::ArchiveNotFound {
                    index,
                    archive: ArchiveId::new(0),
                })?;

        let file_hash = name_hash(file_name);
        let (file_id, _) =
            archive_entry
                .find_file_by_name(file_hash)
                .ok_or_else(|| CacheError::FileNotFound {
                    archive: archive_id,
                    file: FileId::new(0),
                })?;

        self.read_file(index, archive_id, file_id)
    }
}

impl CacheBuilder {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            preload_references: false,
        }
    }

    pub fn preload_references(mut self, preload: bool) -> Self {
        self.preload_references = preload;
        self
    }

    pub fn open(self) -> CacheResult<Cache> {
        let cache = Cache::open(&self.path)?;
        if self.preload_references {
            for index in cache.indices().collect::<Vec<_>>() {
                let _ = cache.reference_table(index);
            }
        }

        Ok(cache)
    }
}
