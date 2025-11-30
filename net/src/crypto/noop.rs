use crate::crypto::StreamCipher;

pub struct NoopCipher;

impl StreamCipher for NoopCipher {
    fn next_u8(&mut self) -> u8 {
        0
    }
}
