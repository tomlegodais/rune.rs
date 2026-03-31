use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

use super::{InboundDecoder, IncomingMessage};

pub struct MoveClick {
    pub x: u16,
    pub y: u16,
    pub ctrl_run: bool,
}

const _: () = {
    const OPCODE: u8 = 5;

    #[message_decoder]
    fn decode_gameclick(mut payload: Bytes) -> IncomingMessage {
        let ctrl_run = payload.get_u8() == 1;
        let x = payload.get_u16();
        let y = payload.get_u16();
        Box::new(MoveClick { x, y, ctrl_run })
    }
};

const _: () = {
    const OPCODE: u8 = 59;

    #[message_decoder]
    fn decode_minimapclick(mut payload: Bytes) -> IncomingMessage {
        let ctrl_run = payload.get_u8() == 1;
        let x = payload.get_u16();
        let y = payload.get_u16();
        Box::new(MoveClick { x, y, ctrl_run })
    }
};
