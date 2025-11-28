use crate::error::SessionError;
use crate::message::{FileRequest, Js5Inbound, Js5Outbound, RequestOpcode};
use filesystem::{ArchiveId, IndexId};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Default)]
pub struct Js5Codec;

impl Decoder for Js5Codec {
    type Item = Js5Inbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if src.len() < 4 {
                return Ok(None);
            }

            let opcode = src.get_u8();
            let request_opcode = RequestOpcode::try_from(opcode)
                .map_err(|_| SessionError::InvalidRequestOpcode(opcode))?;

            match request_opcode {
                RequestOpcode::Normal | RequestOpcode::Urgent => {
                    let urgent = request_opcode == RequestOpcode::Urgent;
                    let index_id = src.get_u8();
                    let archive_id = src.get_u16() as u32;
                    let request = FileRequest::new(
                        urgent,
                        IndexId::new(index_id),
                        ArchiveId::new(archive_id),
                    );

                    return Ok(Some(Js5Inbound::FileRequest(request)));
                }

                RequestOpcode::EncryptionKey => {
                    let key = src.get_u8();
                    src.advance(2);

                    return Ok(Some(Js5Inbound::EncryptionKey(key)));
                }

                _ => src.advance(3),
            }
        }
    }
}

impl Encoder<Js5Outbound> for Js5Codec {
    type Error = SessionError;

    fn encode(&mut self, item: Js5Outbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.data.is_empty() {
            return Ok(());
        }

        let compression = item.data[0];
        let compression_byte = if item.urgent {
            compression
        } else {
            compression | 0x80
        };

        let container_data = &item.data[1..];
        let data_len = container_data.len();
        let num_markers = if data_len <= 508 {
            0
        } else {
            1 + (data_len - 508 - 1) / 511
        };

        let total_size = 4 + data_len + num_markers;
        dst.reserve(total_size);
        dst.put_u8(item.index.as_u8());
        dst.put_u16(item.archive.as_u32() as u16);
        dst.put_u8(compression_byte);

        let first_chunk_size = container_data.len().min(508);
        dst.extend_from_slice(&container_data[..first_chunk_size]);

        let mut offset = first_chunk_size;
        while offset < container_data.len() {
            dst.put_u8(0xFF);

            let chunk_size = (container_data.len() - offset).min(511);
            dst.extend_from_slice(&container_data[offset..offset + chunk_size]);
            offset += chunk_size;
        }

        Ok(())
    }
}
