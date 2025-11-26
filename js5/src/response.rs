use filesystem::{ArchiveId, IndexId};

const BLOCK_MARKER: u8 = 0xFF;

pub struct ResponseEncoder;

impl ResponseEncoder {
    pub fn encode(index: IndexId, archive: ArchiveId, data: &[u8], urgent: bool) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let compression = data[0];
        let compression_byte = if urgent {
            compression
        } else {
            compression | 0x80
        };

        let container_data = &data[1..];
        let data_len = container_data.len();
        let num_markers = if data_len <= 508 {
            0
        } else {
            1 + (data_len - 508 - 1) / 511
        };

        let total_size = 4 + data_len + num_markers;
        let mut result = Vec::with_capacity(total_size);

        result.push(index.as_u8());
        result.extend_from_slice(&(archive.as_u32() as u16).to_be_bytes());
        result.push(compression_byte);

        let first_chunk_size = container_data.len().min(508);
        result.extend_from_slice(&container_data[..first_chunk_size]);

        let mut offset = first_chunk_size;
        while offset < container_data.len() {
            result.push(BLOCK_MARKER);

            let chunk_size = (container_data.len() - offset).min(511);
            result.extend_from_slice(&container_data[offset..offset + chunk_size]);
            offset += chunk_size;
        }

        result
    }
}
