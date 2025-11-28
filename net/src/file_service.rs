use crate::request::FileRequest;
use crate::response::FileResponseEncoder;
use filesystem::{Cache, CacheResult};
use std::sync::Arc;

pub struct FileService {
    cache: Arc<Cache>,
    checksum: Vec<u8>,
}

impl FileService {
    pub fn new(cache: Arc<Cache>) -> anyhow::Result<Self> {
        let checksum = filesystem::build_checksum_table(&cache)?;
        Ok(Self { cache, checksum })
    }

    pub fn serve(&self, request: &FileRequest) -> CacheResult<Vec<u8>> {
        let data = self.get_file_data(request)?;
        Ok(FileResponseEncoder::encode(
            request.index,
            request.archive,
            &data,
            request.urgent,
        ))
    }

    pub fn get_file_data(&self, request: &FileRequest) -> CacheResult<Vec<u8>> {
        if request.index.is_reference() && request.archive.is_reference() {
            return Ok(self.checksum.clone());
        }

        let mut data = self.cache.read_archive_raw(request.index, request.archive)?;
        if !request.index.is_reference() && data.len() >= 2 {
            data.truncate(data.len() - 2);
        }

        Ok(data)
    }
}
