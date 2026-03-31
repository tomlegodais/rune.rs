use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};
use crate::inbound::Op;

pub struct OpNpc {
    pub op: Op,
    pub npc_index: u16,
    pub ctrl_run: bool,
}

const _: () = {
    const OPCODE: u8 = 13;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let ctrl_run = payload.get_u8_sub() == 1;
        let npc_index = payload.get_u16();
        Box::new(OpNpc {
            op: Op::Op1,
            npc_index,
            ctrl_run,
        })
    }
};

const _: () = {
    const OPCODE: u8 = 30;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let ctrl_run = payload.get_u8_sub() == 1;
        let npc_index = payload.get_u16_le();
        Box::new(OpNpc {
            op: Op::Op2,
            npc_index,
            ctrl_run,
        })
    }
};

const _: () = {
    const OPCODE: u8 = 31;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let npc_index = payload.get_u16_le_add();
        let ctrl_run = payload.get_u8_sub() == 1;
        Box::new(OpNpc {
            op: Op::Op3,
            npc_index,
            ctrl_run,
        })
    }
};
