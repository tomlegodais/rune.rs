use net::{GameMessage, MessageType, ServerMessage};
use tokio_util::bytes::{BufMut, BytesMut};

pub struct RootInterface {
    pub root_id: u16,
}

impl ServerMessage for RootInterface {
    fn into_game_message(self) -> GameMessage {
        let mut buf = BytesMut::new();
        buf.put_u8(0);
        buf.put_u16_le(0);
        buf.put_u16_le(self.root_id);

        GameMessage {
            opcode: 102,
            ty: MessageType::Fixed,
            payload: buf.freeze(),
        }
    }
}
