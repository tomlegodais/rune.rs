use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};

pub struct OpNpcT {
    pub npc_index: u16,
    pub ctrl_run: bool,
}

const _: () = {
    const OPCODE: u8 = 35;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let ctrl_run = payload.get_u8_sub() == 1;
        let npc_index = payload.get_u16();
        Box::new(OpNpcT { npc_index, ctrl_run })
    }
};
