use crate::error::SessionError;
use filesystem::{ArchiveId, IndexId};
use num_enum::TryFromPrimitive;
use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct FileRequest {
    pub urgent: bool,
    pub index: IndexId,
    pub archive: ArchiveId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
enum RequestOpcode {
    Normal = 0,
    Urgent = 1,
    LoggedIn = 2,
    LoggedOut = 3,
    EncryptionKey = 4,
    Connected = 6,
    Disconnected = 7,
}

#[derive(Debug)]
pub enum Js5Inbound {
    FileRequest(FileRequest),
    EncryptionKey(u8),
}

#[derive(Debug)]
pub struct Js5Outbound {
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct Js5Codec {
    pub xor_key: u8,
}

impl Js5Codec {
    pub fn new() -> Self {
        Self { xor_key: 0 }
    }
}

impl FileRequest {
    pub fn new(urgent: bool, index: IndexId, archive: ArchiveId) -> Self {
        Self {
            urgent,
            index,
            archive,
        }
    }

    pub fn parse(urgent: bool, data: &[u8]) -> Self {
        let index = IndexId::new(data[0]);
        let archive = ArchiveId::new(u16::from_be_bytes([data[1], data[2]]) as u32);
        Self::new(urgent, index, archive)
    }
}

impl Decoder for Js5Codec {
    type Item = Js5Inbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            let opcode = src.get_u8();

            println!("opcode: {}", opcode);

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

                    println!("request: {:?}", request);

                    return Ok(Some(Js5Inbound::FileRequest(request)));
                }

                RequestOpcode::EncryptionKey => {
                    let key = src.get_u8();
                    src.advance(2); // skip 2 bytes

                    return Ok(Some(Js5Inbound::EncryptionKey(key)));
                }

                _ => src.advance(3), // skip 3 bytes
            }
        }
    }
}

impl Encoder<Js5Outbound> for Js5Codec {
    type Error = SessionError;

    fn encode(&mut self, item: Js5Outbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if self.xor_key == 0 {
            dst.extend_from_slice(&item.bytes);
        } else {
            dst.reserve(item.bytes.len());
            for b in item.bytes {
                dst.extend_from_slice(&[b ^ self.xor_key]);
            }
        }

        Ok(())
    }
}
