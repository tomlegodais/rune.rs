use filesystem::{ArchiveId, IndexId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RequestOpcode {
    FileRequestNormal = 0,
    FileRequestUrgent = 1,
    LoggedIn = 2,
    LoggedOut = 3,
    EncryptionKeys = 4,
    Connected = 6,
    Disconnected = 7,
}

impl RequestOpcode {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::FileRequestNormal),
            1 => Some(Self::FileRequestUrgent),
            2 => Some(Self::LoggedIn),
            3 => Some(Self::LoggedOut),
            4 => Some(Self::EncryptionKeys),
            6 => Some(Self::Connected),
            7 => Some(Self::Disconnected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn parse(urgent: bool, data: &[u8; 3]) -> Self {
        let index = IndexId::new(data[0]);
        let archive = ArchiveId::new(u16::from_be_bytes([data[1], data[2]]) as u32);
        Self::new(urgent, index, archive)
    }
}
