use crate::error::{CacheError, CacheResult};
use crate::id::FileId;
use std::collections::BTreeMap;

pub fn unpack_archive<'a>(
    data: &'a [u8],
    file_ids: &[FileId],
) -> CacheResult<BTreeMap<FileId, &'a [u8]>> {
    let file_count = file_ids.len();
    if file_count == 1 {
        let mut result = BTreeMap::new();
        result.insert(file_ids[0], data);
        return Ok(result);
    }

    if data.is_empty() {
        return Err(CacheError::InvalidContainer("archive data is empty".into()));
    }

    let chunks = data[data.len() - 1] as usize;
    let trailer_size = file_count * chunks * 4 + 1;
    if trailer_size > data.len() {
        return Err(CacheError::InvalidContainer(
            "archive trailer larger than archive data".into(),
        ));
    }

    let trailer_start = data.len() - trailer_size;
    let trailer = &data[trailer_start..data.len() - 1];
    let mut file_sizes = vec![0u32; file_count];
    let mut trailer_pos = 0;

    for _chunk in 0..chunks {
        let mut accumulator: i32 = 0;
        for file_idx in 0..file_count {
            if trailer_pos + 4 > trailer.len() {
                return Err(CacheError::InvalidContainer("trailer read past end".into()));
            }

            let delta = i32::from_be_bytes([
                trailer[trailer_pos],
                trailer[trailer_pos + 1],
                trailer[trailer_pos + 2],
                trailer[trailer_pos + 3],
            ]);
            trailer_pos += 4;

            accumulator = accumulator.wrapping_add(delta);
            file_sizes[file_idx] = file_sizes[file_idx].wrapping_add(accumulator as u32);
        }
    }

    let mut result = BTreeMap::new();
    let mut offset = 0usize;

    for (idx, &file_id) in file_ids.iter().enumerate() {
        let size = file_sizes[idx] as usize;
        let end = offset + size;
        if end > trailer_start {
            return Err(CacheError::InvalidContainer(
                "file extends into trailer".into(),
            ));
        }

        result.insert(file_id, &data[offset..end]);
        offset = end;
    }

    Ok(result)
}

pub fn unpack_archive_owned(
    data: &[u8],
    file_ids: &[FileId],
) -> CacheResult<BTreeMap<FileId, Vec<u8>>> {
    unpack_archive(data, file_ids)
        .map(|map| map.into_iter().map(|(k, v)| (k, v.to_vec())).collect())
}
