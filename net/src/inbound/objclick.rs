use super::{InboundDecoder, IncomingMessage};
use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

pub struct ObjClick {
    pub item_id: u16,
    pub force_run: bool,
    pub x: u16,
    pub y: u16,
}

const _: () = {
    const OPCODE: u8 = 80;

    #[message_decoder]
    fn decode_obj_click(mut payload: Bytes) -> IncomingMessage {
        let item_id = payload.get_u16_add();
        let force_run = payload.get_u8_neg() != 0;
        let x = payload.get_u16();
        let y = payload.get_u16_le_add();

        Box::new(ObjClick {
            item_id,
            force_run,
            x,
            y,
        })
    }
};
