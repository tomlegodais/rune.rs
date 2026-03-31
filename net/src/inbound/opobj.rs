use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};

pub struct OpObj {
    pub obj_id: u16,
    pub ctrl_run: bool,
    pub x: u16,
    pub y: u16,
}

const _: () = {
    const OPCODE: u8 = 80;

    #[message_decoder]
    fn decode_obj_click(mut payload: Bytes) -> IncomingMessage {
        let obj_id = payload.get_u16_add();
        let ctrl_run = payload.get_u8_neg() != 0;
        let x = payload.get_u16();
        let y = payload.get_u16_le_add();

        Box::new(OpObj {
            obj_id,
            ctrl_run,
            x,
            y,
        })
    }
};
