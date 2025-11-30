use tokio_util::bytes::Bytes;

#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Fixed,
    Byte,
    Short,
}

#[derive(Debug)]
pub struct GameMessage {
    pub opcode: u8,
    pub ty: MessageType,
    pub payload: Bytes,
}

pub trait ServerMessage {
    fn into_game_message(self) -> GameMessage;
}
