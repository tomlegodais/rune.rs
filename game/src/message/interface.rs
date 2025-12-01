use crate::player::RootWidget;
use net::{GameMessage, MessageType, ServerMessage};
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

pub struct SetRootWidget(pub RootWidget);

impl ServerMessage for SetRootWidget {
    fn into_game_message(self) -> GameMessage {
        let mut buf = BytesMut::new();
        buf.put_u8(0);
        buf.put_u16_le(0);
        buf.put_u16_le(self.0.0);

        GameMessage {
            opcode: 102,
            ty: MessageType::Fixed,
            payload: buf.freeze(),
        }
    }
}

pub struct OpenWidget {
    pub parent: u16,
    pub position: u16,
    pub interface: u16,
    pub click_through: bool,
}

impl ServerMessage for OpenWidget {
    fn into_game_message(self) -> GameMessage {
        let mut buf = BytesMut::new();
        buf.put_u8_sub(self.click_through as u8);
        buf.put_u16_add(0);
        buf.put_u32(self.to_hash());
        buf.put_u16_le(self.interface);

        GameMessage {
            opcode: 22,
            ty: MessageType::Fixed,
            payload: buf.freeze(),
        }
    }
}

impl OpenWidget {
    pub fn to_hash(&self) -> u32 {
        ((self.parent as u32) << 16) | (self.position as u32)
    }
}
