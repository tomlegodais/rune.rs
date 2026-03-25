use super::{InboundDecoder, IncomingMessage};
use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

#[derive(Debug)]
pub struct PublicChat {
    pub color: u8,
    pub effect: u8,
    pub text_len: usize,
    pub payload: Vec<u8>,
}

const OPCODE: u8 = 79;

#[message_decoder]
fn decode(mut payload: Bytes) -> IncomingMessage {
    let _script = payload.get_u8();
    let color = payload.get_u8();
    let effect = payload.get_u8();
    let text_len = payload.get_smart() as usize;

    Box::new(PublicChat {
        color,
        effect,
        text_len,
        payload: payload.to_vec(),
    })
}
