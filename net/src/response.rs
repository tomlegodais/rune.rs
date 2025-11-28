use filesystem::{ArchiveId, IndexId};

const BLOCK_MARKER: u8 = 0xFF;

pub struct FileResponseEncoder;
pub struct WorldListEncoder;

impl FileResponseEncoder {
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

impl WorldListEncoder {
    pub fn encode(full_update: bool, host: &str, player_count: u16) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(128);

        buffer.push(1);
        buffer.push(if full_update { 1 } else { 0 });

        if full_update {
            write_smart(&mut buffer, 1); // country count
            write_smart(&mut buffer, 0); // country id
            write_jag_string(&mut buffer, "World 1"); // world name

            write_smart(&mut buffer, 0); // min world id
            write_smart(&mut buffer, 2); // max world id + 1
            write_smart(&mut buffer, 1); // world count

            write_smart(&mut buffer, 1); // world id
            buffer.push(0); // location (country index)
            buffer.extend_from_slice(&(0x1u32 | 0x8).to_be_bytes()); // flags: members | lootshare
            write_jag_string(&mut buffer, ""); // activity
            write_jag_string(&mut buffer, host); // hostname
            buffer.extend_from_slice(&0x94DA4A87u32.to_be_bytes()); // session id
        }

        write_smart(&mut buffer, 1); // world count
        buffer.extend_from_slice(&player_count.to_be_bytes()); // player count

        let mut response = Vec::with_capacity(3 + buffer.len());
        response.push(0);
        response.extend_from_slice(&(buffer.len() as u16).to_be_bytes());
        response.extend_from_slice(&buffer);

        response
    }
}

fn write_smart(buffer: &mut Vec<u8>, value: u16) {
    if value < 128 {
        buffer.push(value as u8);
    } else {
        buffer.push(((value >> 8) as u8) | 0x80);
        buffer.push(value as u8);
    }
}

fn write_jag_string(buffer: &mut Vec<u8>, s: &str) {
    buffer.push(0);
    buffer.extend_from_slice(s.as_bytes());
    buffer.push(0);
}
