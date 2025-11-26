use crate::{ArchiveId, Cache, CacheResult, IndexId, REFERENCE_INDEX, crc32};

pub fn build_checksum_table(cache: &Cache) -> CacheResult<Vec<u8>> {
    let index_count = (IndexId::MAX_INDEX as usize) + 1;
    let mut table = Vec::with_capacity(index_count * 8);

    for i in 0..index_count {
        let index = IndexId::new(i as u8);

        match cache.read_archive_raw(REFERENCE_INDEX, ArchiveId::new(i as u32)) {
            Ok(data) => {
                let crc = crc32(&data);
                let version = cache
                    .reference_table(index)
                    .map(|rt| rt.version.unwrap_or(0))
                    .unwrap_or(0);

                table.extend_from_slice(&crc.to_be_bytes());
                table.extend_from_slice(&version.to_be_bytes());
            }
            Err(_) => {
                table.extend_from_slice(&[0u8; 8]);
            }
        }
    }

    let mut response = Vec::with_capacity(5 + table.len());
    response.push(0);
    response.extend_from_slice(&(table.len() as u32).to_be_bytes());
    response.extend_from_slice(&table);

    Ok(response)
}
