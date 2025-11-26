use crate::request::FileRequest;
use crate::response::ResponseEncoder;
use filesystem::{Cache, CacheResult};

pub struct Js5Service {
    cache: Cache,
    checksum: Vec<u8>,
}

impl Js5Service {
    pub fn new(cache: Cache) -> CacheResult<Self> {
        let checksum = filesystem::build_checksum_table(&cache)?;
        Ok(Self { cache, checksum })
    }

    pub fn serve(&self, request: &FileRequest) -> CacheResult<Vec<u8>> {
        let data = self.get_file_data(request)?;
        Ok(ResponseEncoder::encode(
            request.index,
            request.archive,
            &data,
            request.urgent,
        ))
    }

    pub fn get_file_data(&self, request: &FileRequest) -> CacheResult<Vec<u8>> {
        if request.index.is_reference() && request.archive.as_u32() == 255 {
            return Ok(self.checksum.clone());
        }

        let mut data = self.cache.read_archive_raw(request.index, request.archive)?;
        if !request.index.is_reference() {
            data.truncate(data.len() - 2);
        }

        Ok(data)
    }
}
