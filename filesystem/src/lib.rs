mod archive;
mod cache;
mod checksum_table;
mod codec;
pub mod definition;
mod error;
mod id;
pub mod loader;
mod reference;
mod store;

pub use cache::{Cache, CacheBuilder};
pub use checksum_table::build_checksum_table;
pub use codec::Compression;
pub use error::{CacheError, CacheResult};
pub use id::{ArchiveId, FileId, IndexId, REFERENCE_INDEX};
pub use reference::{ArchiveEntry, FileEntry, ReferenceTable, name_hash};

pub mod prelude {
    pub use crate::{
        ArchiveId, Cache, CacheBuilder, CacheError, CacheResult, FileId, IndexId, REFERENCE_INDEX,
        ReferenceTable,
    };
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;

    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }

    !crc
}
