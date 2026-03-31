use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};
use crate::inbound::Op;

pub struct OpPlayer {
    pub op: Op,
    pub player_index: u16,
    pub ctrl_run: bool,
}

const _: () = {
    const OPCODE: u8 = 40;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let player_index = payload.get_u16_add();
        let ctrl_run = payload.get_u8() == 1;
        Box::new(OpPlayer {
            op: Op::Op1,
            player_index,
            ctrl_run,
        })
    }
};

const _: () = {
    const OPCODE: u8 = 41;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let player_index = payload.get_u16_add();
        let ctrl_run = payload.get_u8() == 1;
        Box::new(OpPlayer {
            op: Op::Op2,
            player_index,
            ctrl_run,
        })
    }
};

const _: () = {
    const OPCODE: u8 = 65;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let ctrl_run = payload.get_u8() == 1;
        let player_index = payload.get_u16_le_add();
        Box::new(OpPlayer {
            op: Op::Op4,
            player_index,
            ctrl_run,
        })
    }
};
