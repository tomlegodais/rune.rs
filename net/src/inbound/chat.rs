use super::{InboundDecoder, IncomingMessage};
use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::{BufExt, Huffman};

#[derive(Debug)]
pub struct PublicChat {
    pub color: u8,
    pub effect: u8,
    pub message: String,
}

const OPCODE: u8 = 79;

#[message_decoder]
fn decode(mut payload: Bytes) -> IncomingMessage {
    let _script = payload.get_u8();
    let color = payload.get_u8();
    let effect = payload.get_u8();

    let text_len = payload.get_smart() as usize;
    let message = Huffman::decode(&payload, text_len);

    Box::new(PublicChat {
        color,
        effect,
        message,
    })
}
