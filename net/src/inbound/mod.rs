mod client_cheat;
mod examloc;
mod ifbutton;
mod ifdialogcontinue;
mod ifmoveslot;
mod ifsubclosed;
mod message_public;
mod moveclick;
mod oploc;
mod opnpc;
mod opobj;
mod opplayer;

use std::any::Any;

pub use client_cheat::ClientCheat;
pub use examloc::ExamLoc;
pub use ifbutton::IfButton;
pub use ifdialogcontinue::IfDialogContinue;
pub use ifmoveslot::IfMoveSlot;
pub use ifsubclosed::IfSubClosed;
pub use message_public::MessagePublic;
pub use moveclick::MoveClick;
pub use oploc::OpLoc;
pub use opnpc::OpNpc;
pub use opobj::OpObj;
pub use opplayer::OpPlayer;
use tokio::sync::mpsc;
use tokio_util::bytes::Bytes;

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

const IGNORED_OPCODES: [u8; 10] = [
    74, // Keep Alive
    42, // Mouse Movement
    49, // Mouse Click
    58, // Screen
    17, // Region Loaded
    51, // Move Camera
    71, // Packet Count
    19, // AFK
    21, // Key Typed
    48, // Window Focus
];

pub fn decode(frame: Frame) -> Option<IncomingMessage> {
    match DECODERS[frame.opcode as usize] {
        Some(decode) => Some(decode(frame.payload)),
        None if IGNORED_OPCODES.contains(&frame.opcode) => None,
        None => {
            tracing::debug!(opcode = frame.opcode, "Unhandled Frame");
            None
        }
    }
}
