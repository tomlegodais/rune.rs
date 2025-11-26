use crate::id::{ArchiveId, FileId, IndexId};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("cache directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("data file not found: {0}")]
    DataFileNotFound(PathBuf),

    #[error("index file not found: {index} at {path}")]
    IndexFileNotFound { index: IndexId, path: PathBuf },

    #[error("index {0} does not exist in this cache")]
    IndexNotExists(IndexId),

    #[error("archive {archive} not found in index {index}")]
    ArchiveNotFound { index: IndexId, archive: ArchiveId },

    #[error("file {file} not found in archive {archive}")]
    FileNotFound { archive: ArchiveId, file: FileId },

    #[error("invalid index entry at archive {0}: entry points outside data file")]
    InvalidIndexEntry(ArchiveId),

    #[error("data block chain corrupted for archive {0}: unexpected end")]
    CorruptedBlockChain(ArchiveId),

    #[error("block header mismatch: expected archive {expected}, got {actual}")]
    BlockHeaderMismatch { expected: ArchiveId, actual: u32 },

    #[error("unsupported compression type: {0}")]
    UnsupportedCompression(u8),

    #[error("decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("invalid container format: {0}")]
    InvalidContainer(String),

    #[error("reference table parse error: {0}")]
    ReferenceTableError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type CacheResult<T> = Result<T, CacheError>;
