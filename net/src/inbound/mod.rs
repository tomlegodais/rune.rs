pub mod command;

use crate::Frame;
use std::any::Any;
use tokio::sync::mpsc;
use tokio_util::bytes::Bytes;
use tracing::debug;

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

pub fn decode(frame: Frame) -> Option<IncomingMessage> {
    match DECODERS[frame.opcode as usize] {
        Some(decode) => Some(decode(frame.payload)),
        None => {
            debug!("Unhandled opcode: {}", frame.opcode);
            None
        }
    }
}
