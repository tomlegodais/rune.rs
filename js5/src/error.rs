#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid handshake opcode")]
    InvalidHandshake,

    #[error("client connected to wrong service (expected JS5, got login)")]
    WrongService,

    #[error("client version mismatch")]
    VersionMismatch,

    #[error("invalid opcode: {0}")]
    InvalidOpcode(u8),

    #[error("task panic: {0}")]
    TaskPanic(String),
}
