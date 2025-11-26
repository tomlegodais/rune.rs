use crate::error::{CacheError, CacheResult};
use crate::id::{ArchiveId, IndexId};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

const INDEX_ENTRY_SIZE: usize = 6;
const SECTOR_SIZE: usize = 520;
const SECTOR_HEADER_NORMAL: usize = 8;
const SECTOR_HEADER_EXTENDED: usize = 10;
const EXTENDED_ARCHIVE_THRESHOLD: u32 = 0xFFFF;

pub struct DataStore {
    mmap: Mmap,
}

impl DataStore {
    pub fn open(path: impl AsRef<Path>) -> CacheResult<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|_| CacheError::DataFileNotFound(path.to_owned()))?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn read_archive(
        &self,
        index: IndexId,
        archive: ArchiveId,
        sector: u32,
        size: u32,
    ) -> CacheResult<Vec<u8>> {
        let mut data = Vec::with_capacity(size as usize);
        let extended = archive.as_u32() > EXTENDED_ARCHIVE_THRESHOLD;
        let header_size = if extended {
            SECTOR_HEADER_EXTENDED
        } else {
            SECTOR_HEADER_NORMAL
        };

        let data_per_sector = SECTOR_SIZE - header_size;
        let mut current_sector = sector;
        let mut remaining = size as usize;
        let mut expected_chunk = 0u16;

        while remaining > 0 {
            let offset = current_sector as usize * SECTOR_SIZE;
            if offset + SECTOR_SIZE > self.mmap.len() {
                return Err(CacheError::CorruptedBlockChain(archive));
            }

            let sector_data = &self.mmap[offset..offset + SECTOR_SIZE];
            let (header_archive, header_chunk, next_sector, header_index) = if extended {
                let archive_id = u32::from_be_bytes([
                    sector_data[0],
                    sector_data[1],
                    sector_data[2],
                    sector_data[3],
                ]);
                let chunk = u16::from_be_bytes([sector_data[4], sector_data[5]]);
                let next = u32::from_be_bytes([0, sector_data[6], sector_data[7], sector_data[8]]);
                let idx = sector_data[9];
                (archive_id, chunk, next, idx)
            } else {
                let archive_id = u16::from_be_bytes([sector_data[0], sector_data[1]]) as u32;
                let chunk = u16::from_be_bytes([sector_data[2], sector_data[3]]);
                let next = u32::from_be_bytes([0, sector_data[4], sector_data[5], sector_data[6]]);
                let idx = sector_data[7];
                (archive_id, chunk, next, idx)
            };

            if header_archive != archive.as_u32() {
                return Err(CacheError::BlockHeaderMismatch {
                    expected: archive,
                    actual: header_archive,
                });
            }

            if header_chunk != expected_chunk {
                return Err(CacheError::CorruptedBlockChain(archive));
            }

            if header_index != index.as_u8() {
                return Err(CacheError::CorruptedBlockChain(archive));
            }

            let to_read = remaining.min(data_per_sector);
            data.extend_from_slice(&sector_data[header_size..header_size + to_read]);

            remaining -= to_read;
            current_sector = next_sector;
            expected_chunk += 1;
        }

        Ok(data)
    }
}

pub struct IndexStore {
    mmap: Mmap,
}

impl IndexStore {
    pub fn open(path: impl AsRef<Path>, index_id: IndexId) -> CacheResult<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|_| CacheError::IndexFileNotFound {
            index: index_id,
            path: path.to_owned(),
        })?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn get_entry(&self, archive: ArchiveId) -> Option<(u32, u32)> {
        let offset = archive.as_u32() as usize * INDEX_ENTRY_SIZE;
        if offset + INDEX_ENTRY_SIZE > self.mmap.len() {
            return None;
        }

        let entry = &self.mmap[offset..offset + INDEX_ENTRY_SIZE];
        let size = u32::from_be_bytes([0, entry[0], entry[1], entry[2]]);
        let sector = u32::from_be_bytes([0, entry[3], entry[4], entry[5]]);
        if size == 0 || sector == 0 {
            return None;
        }

        Some((size, sector))
    }

    #[inline(always)]
    pub fn archive_count(&self) -> u32 {
        (self.mmap.len() / INDEX_ENTRY_SIZE) as u32
    }
}
