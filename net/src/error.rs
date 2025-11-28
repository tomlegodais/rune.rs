use std::io;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid handshake opcode: {0}")]
    InvalidHandshakeOpcode(u8),

    #[error("client version mismatch")]
    VersionMismatch,

    #[error("invalid request opcode: {0}")]
    InvalidRequestOpcode(u8),
}

impl SessionError {
    pub fn is_disconnect(&self) -> bool {
        use io::ErrorKind::*;

        match self {
            SessionError::Io(e) => {
                matches!(e.kind(), ConnectionReset | ConnectionAborted | BrokenPipe)
            }
            _ => false,
        }
    }
}
