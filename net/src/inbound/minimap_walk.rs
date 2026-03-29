use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

use super::{InboundDecoder, IncomingMessage};
use crate::inbound::walk::WalkRequest;

const OPCODE: u8 = 59;

#[message_decoder]
fn decode(mut payload: Bytes) -> IncomingMessage {
    let force_run = payload.get_u8() == 1;
    let x = payload.get_u16();
    let y = payload.get_u16();
    Box::new(WalkRequest { x, y, force_run })
}
