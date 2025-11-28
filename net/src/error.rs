#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid handshake opcode: {0}")]
    InvalidHandshakeOpcode(u8),

    #[error("client version mismatch")]
    VersionMismatch,

    #[error("invalid request opcode: {0}")]
    InvalidRequestOpcode(u8),

    #[error("task panic: {0}")]
    TaskPanic(String),
}
