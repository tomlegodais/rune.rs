use super::{InboundDecoder, IncomingMessage};
use crate::inbound::ClickOption;
use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

pub struct ObjectClick {
    pub option: ClickOption,
    pub id: u16,
    pub x: u16,
    pub y: u16,
    pub ctrl_run: bool,
}

const _: () = {
    const OPCODE: u8 = 77;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let x = payload.get_u16_le_add();
        let ctrl_run = payload.get_u8() == 1;
        let id = payload.get_u16();
        let y = payload.get_u16();
        Box::new(ObjectClick {
            option: ClickOption::One,
            id,
            x,
            y,
            ctrl_run,
        })
    }
};

const _: () = {
    const OPCODE: u8 = 14;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let x = payload.get_u16_add();
        let y = payload.get_u16_add();
        let id = payload.get_u16_le_add();
        let ctrl_run = payload.get_u8() == 1;
        Box::new(ObjectClick {
            option: ClickOption::Two,
            id,
            x,
            y,
            ctrl_run,
        })
    }
};

const _: () = {
    const OPCODE: u8 = 10;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        let x = payload.get_u16_le();
        let ctrl_run = payload.get_u8() == 1;
        let y = payload.get_u16();
        let id = payload.get_u16_le_add();
        Box::new(ObjectClick {
            option: ClickOption::Three,
            id,
            x,
            y,
            ctrl_run,
        })
    }
};
