pub trait StreamCipher: Send {
    fn next_u8(&mut self) -> u8;
}
