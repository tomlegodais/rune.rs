mod archive;
mod cache;
mod checksum_table;
mod codec;
mod config;
mod error;
mod id;
mod loader;

pub use config::{
    AmmoType, AttackType, CombatStyle, EnumType, EnumValue, EquipBonuses, LocType, NpcType, ObjType, ParamMap,
    ParamValue, StructType, StyleName, TransformKind, VarbitType, WeaponCategory, WeaponStance, WearFlag, WearPos,
    XpType,
};
pub use loader::{EnumLoader, LocLoader, NpcLoader, ObjLoader, StructLoader, VarbitLoader};
mod reference;
mod store;

pub use cache::{Cache, CacheBuilder};
pub use checksum_table::build_checksum_table;
pub use codec::Compression;
pub use error::{CacheError, CacheResult};
pub use id::{ArchiveId, FileId, IndexId, REFERENCE_INDEX};
pub use reference::{ArchiveEntry, FileEntry, ReferenceTable, name_hash};

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
