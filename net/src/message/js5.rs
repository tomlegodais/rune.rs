use filesystem::{ArchiveId, IndexId};
use num_enum::TryFromPrimitive;
use std::cmp::Ordering;

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

#[derive(Debug)]
pub struct PriorityRequest {
    pub request: FileRequest,
    pub sequence: u64,
}

impl PartialEq for PriorityRequest {
    fn eq(&self, other: &Self) -> bool {
        self.request.urgent == other.request.urgent && self.sequence == other.sequence
    }
}

impl Eq for PriorityRequest {}
impl Ord for PriorityRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.request.urgent, other.request.urgent) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => other.sequence.cmp(&self.sequence),
        }
    }
}

impl PartialOrd for PriorityRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
