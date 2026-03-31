mod button;
mod chat;
mod command;
mod ifmoveslot;
mod loc;
mod minimap_walk;
mod npc;
mod objclick;
mod player;
mod walk;

use std::any::Any;

pub use button::ButtonClick;
pub use chat::PublicChat;
pub use command::ClientCommand;
pub use ifmoveslot::IfMoveSlot;
pub use loc::LocClick;
pub use npc::NpcClick;
pub use objclick::ObjClick;
pub use player::PlayerClick;
use tokio::sync::mpsc;
use tokio_util::bytes::Bytes;
use tracing::debug;
pub use walk::WalkRequest;

use crate::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Op1,
    Op2,
    Op3,
    Op4,
    Op5,
    Op6,
    Op7,
    Op8,
    Op9,
    Op10,
}

pub type IncomingMessage = Box<dyn Any + Send>;

pub type Inbox = mpsc::Receiver<IncomingMessage>;

pub trait InboxExt {
    fn try_recv_all(&mut self) -> Vec<IncomingMessage>;
}

impl InboxExt for Inbox {
    fn try_recv_all(&mut self) -> Vec<IncomingMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.try_recv() {
            messages.push(msg);
        }
        messages
    }
}

type DecodeFn = fn(Bytes) -> IncomingMessage;

pub struct InboundDecoder {
    pub opcode: u8,
    pub decode: DecodeFn,
}

inventory::collect!(InboundDecoder);

static DECODERS: std::sync::LazyLock<[Option<DecodeFn>; 256]> = std::sync::LazyLock::new(|| {
    let mut table = [None; 256];
    for entry in inventory::iter::<InboundDecoder> {
        table[entry.opcode as usize] = Some(entry.decode);
    }
    table
});

const IGNORED_OPCODES: [u8; 3] = [
    74, // Keep Alive
    42, // Mouse Movement
    49, // Mouse Click
];

pub fn decode(frame: Frame) -> Option<IncomingMessage> {
    match DECODERS[frame.opcode as usize] {
        Some(decode) => Some(decode(frame.payload)),
        None if IGNORED_OPCODES.contains(&frame.opcode) => None,
        None => {
            debug!("Unhandled message opcode: {}", frame.opcode);
            None
        }
    }
}
