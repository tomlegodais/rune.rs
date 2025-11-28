use filesystem::{ArchiveId, IndexId};
use num_enum::TryFromPrimitive;

#[derive(Debug)]
pub struct FileRequest {
    pub urgent: bool,
    pub index: IndexId,
    pub archive: ArchiveId,
}

impl FileRequest {
    pub fn new(urgent: bool, index: IndexId, archive: ArchiveId) -> Self {
        Self {
            urgent,
            index,
            archive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum RequestOpcode {
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
    pub index: IndexId,
    pub archive: ArchiveId,
    pub data: Vec<u8>,
    pub urgent: bool,
}
