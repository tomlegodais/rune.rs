use crate::error::CacheResult;
use crate::id::{ArchiveId, FileId};
use std::collections::BTreeMap;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

#[derive(Debug, Clone)]
pub struct ReferenceTable {
    pub format: u8,
    pub version: Option<u32>,
    pub flags: u8,
    pub archives: BTreeMap<ArchiveId, ArchiveEntry>,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub name_hash: Option<i32>,
    pub crc: u32,
    pub version: u32,
    pub files: BTreeMap<FileId, FileEntry>,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name_hash: Option<i32>,
}

impl ReferenceTable {
    const FLAG_NAMES: u8 = 0x01;
    const FLAG_WHIRLPOOL: u8 = 0x02;
    const FLAG_SIZES: u8 = 0x04;

    pub fn parse(data: &[u8]) -> CacheResult<Self> {
        let mut buffer = Bytes::copy_from_slice(data);
        let format = buffer.get_u8();
        let version = if format >= 6 {
            Some(buffer.get_u32())
        } else {
            None
        };

        let flags = buffer.get_u8();
        let has_names = flags & Self::FLAG_NAMES != 0;
        let _has_whirlpool = flags & Self::FLAG_WHIRLPOOL != 0;
        let _has_sizes = flags & Self::FLAG_SIZES != 0;
        let archive_count = if format >= 7 {
            buffer.get_smart_u32()
        } else {
            buffer.get_u16() as u32
        };

        let mut archive_ids = Vec::with_capacity(archive_count as usize);
        let mut accumulator: u32 = 0;
        for _ in 0..archive_count {
            let delta = if format >= 7 {
                buffer.get_smart_u32()
            } else {
                buffer.get_u16() as u32
            };
            accumulator += delta;
            archive_ids.push(ArchiveId::new(accumulator));
        }

        let mut archives: BTreeMap<ArchiveId, ArchiveEntry> = archive_ids
            .iter()
            .map(|&id| {
                (
                    id,
                    ArchiveEntry {
                        name_hash: None,
                        crc: 0,
                        version: 0,
                        files: BTreeMap::new(),
                    },
                )
            })
            .collect();

        if has_names {
            for &id in &archive_ids {
                if let Some(entry) = archives.get_mut(&id) {
                    entry.name_hash = Some(buffer.get_i32());
                }
            }
        }

        for &id in &archive_ids {
            if let Some(entry) = archives.get_mut(&id) {
                entry.crc = buffer.get_u32();
            }
        }

        if _has_whirlpool {
            for _ in &archive_ids {
                buffer.advance(64);
            }
        }

        if _has_sizes {
            for _ in &archive_ids {
                buffer.advance(8);
            }
        }

        for &id in &archive_ids {
            if let Some(entry) = archives.get_mut(&id) {
                entry.version = buffer.get_u32();
            }
        }

        let mut file_counts: Vec<u32> = Vec::with_capacity(archive_ids.len());
        for _ in &archive_ids {
            let count = if format >= 7 {
                buffer.get_smart_u32()
            } else {
                buffer.get_u16() as u32
            };
            file_counts.push(count);
        }

        for (i, &archive_id) in archive_ids.iter().enumerate() {
            let file_count = file_counts[i];
            let mut file_accumulator: u32 = 0;

            if let Some(archive) = archives.get_mut(&archive_id) {
                for _ in 0..file_count {
                    let delta = if format >= 7 {
                        buffer.get_smart_u32()
                    } else {
                        buffer.get_u16() as u32
                    };
                    file_accumulator += delta;
                    let file_id = FileId::new(file_accumulator);
                    archive.files.insert(file_id, FileEntry { name_hash: None });
                }
            }
        }

        if has_names {
            for &archive_id in &archive_ids {
                if let Some(archive) = archives.get_mut(&archive_id) {
                    let file_ids: Vec<FileId> = archive.files.keys().copied().collect();
                    for file_id in file_ids {
                        if let Some(file) = archive.files.get_mut(&file_id) {
                            file.name_hash = Some(buffer.get_i32());
                        }
                    }
                }
            }
        }

        Ok(Self {
            format,
            version,
            flags,
            archives,
        })
    }

    pub fn archive(&self, id: ArchiveId) -> Option<&ArchiveEntry> {
        self.archives.get(&id)
    }

    pub fn iter_archives(&self) -> impl Iterator<Item = (ArchiveId, &ArchiveEntry)> {
        self.archives.iter().map(|(&id, entry)| (id, entry))
    }

    pub fn iter_archive_ids(&self) -> impl Iterator<Item = ArchiveId> + '_ {
        self.archives.keys().copied()
    }

    pub fn find_by_name(&self, hash: i32) -> Option<(ArchiveId, &ArchiveEntry)> {
        self.archives
            .iter()
            .find(|(_, entry)| entry.name_hash == Some(hash))
            .map(|(&id, entry)| (id, entry))
    }
}

impl ArchiveEntry {
    pub fn find_file_by_name(&self, hash: i32) -> Option<(FileId, &FileEntry)> {
        self.files
            .iter()
            .find(|(_, entry)| entry.name_hash == Some(hash))
            .map(|(&id, entry)| (id, entry))
    }
}

pub fn name_hash(name: &str) -> i32 {
    let mut hash: i32 = 0;
    for byte in name.to_lowercase().bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as i32);
    }
    hash
}
