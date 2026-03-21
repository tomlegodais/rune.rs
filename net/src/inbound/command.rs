use super::{InboundDecoder, IncomingMessage};
use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

pub struct ClientCommand {
    pub command: String,
}

const OPCODE: u8 = 78;

#[message_decoder]
fn decode(mut payload: Bytes) -> IncomingMessage {
    let _client_command = payload.get_u8() == 1;
    let command = payload.get_string();
    Box::new(ClientCommand { command })
}
