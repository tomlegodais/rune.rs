use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};

pub struct ClientCommand {
    pub command: String,
    pub client_sent: bool,
}

const OPCODE: u8 = 78;

#[message_decoder]
fn decode(mut payload: Bytes) -> IncomingMessage {
    let client_sent = payload.get_u8() == 1;
    let command = payload.get_string();
    Box::new(ClientCommand { command, client_sent })
}
