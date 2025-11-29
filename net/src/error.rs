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

    #[error("invalid login type: {0}")]
    InvalidLoginType(u8),

    #[error("invalid encryption: {0}")]
    RsaDecrypt(#[from] anyhow::Error),

    #[error("invalid encrypted block type: {0}")]
    InvalidEncryptedType(u8),

    #[error("Username hash mismatch")]
    UsernameHashMismatch,
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
