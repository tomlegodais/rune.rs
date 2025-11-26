use crate::error::{CacheError, CacheResult};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    None = 0,
    Bzip2 = 1,
    Gzip = 2,
}

impl Compression {
    pub fn from_byte(byte: u8) -> CacheResult<Self> {
        match byte {
            0 => Ok(Self::None),
            1 => Ok(Self::Bzip2),
            2 => Ok(Self::Gzip),
            other => Err(CacheError::UnsupportedCompression(other)),
        }
    }
}

pub fn decompress(
    compression: Compression,
    data: &[u8],
    expected_size: usize,
) -> CacheResult<Vec<u8>> {
    match compression {
        Compression::None => Ok(data.to_vec()),
        Compression::Bzip2 => {
            let mut with_header = Vec::with_capacity(data.len() + 4);
            with_header.extend_from_slice(b"BZh1");
            with_header.extend_from_slice(data);

            let mut decoder = BzDecoder::new(&with_header[..]);
            let mut output = Vec::with_capacity(expected_size);
            decoder
                .read_to_end(&mut output)
                .map_err(|e| CacheError::DecompressionFailed(format!("BZIP2: {}", e)))?;
            Ok(output)
        }
        Compression::Gzip => {
            let mut decoder = GzDecoder::new(data);
            let mut output = Vec::with_capacity(expected_size);
            decoder
                .read_to_end(&mut output)
                .map_err(|e| CacheError::DecompressionFailed(format!("GZIP: {}", e)))?;
            Ok(output)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerHeader {
    pub compression: Compression,
    pub compressed_size: u32,
    pub uncompressed_size: Option<u32>,
}

impl ContainerHeader {
    pub fn parse(data: &[u8]) -> CacheResult<(Self, usize)> {
        if data.len() < 5 {
            return Err(CacheError::InvalidContainer(
                "container too small for header".into(),
            ));
        }

        let compression = Compression::from_byte(data[0])?;
        let compressed_size = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        let (uncompressed_size, header_len) = if compression == Compression::None {
            (None, 5)
        } else {
            if data.len() < 9 {
                return Err(CacheError::InvalidContainer(
                    "container too small for compression header".into(),
                ));
            }
            let size = u32::from_be_bytes([data[5], data[6], data[7], data[8]]);
            (Some(size), 9)
        };

        Ok((
            Self {
                compression,
                compressed_size,
                uncompressed_size,
            },
            header_len,
        ))
    }
}

pub fn decode_container(raw: &[u8]) -> CacheResult<Vec<u8>> {
    let (header, data_offset) = ContainerHeader::parse(raw)?;
    let compressed_data = &raw[data_offset..data_offset + header.compressed_size as usize];
    let expected_size = header.uncompressed_size.unwrap_or(header.compressed_size) as usize;

    decompress(header.compression, compressed_data, expected_size)
}
