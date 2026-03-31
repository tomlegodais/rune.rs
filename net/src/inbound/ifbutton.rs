use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

use super::{InboundDecoder, IncomingMessage, Op};

pub struct IfButton {
    pub op: Op,
    pub interface: u16,
    pub component: u16,
    pub slot1: u16,
    pub slot2: u16,
}

fn decode_ifbutton(payload: &mut Bytes, op: Op) -> IfButton {
    let hash = payload.get_u32();
    let slot1 = payload.get_u16_le();
    let slot2 = payload.get_u16_le();
    IfButton {
        op,
        interface: (hash >> 16) as u16,
        component: (hash & 0xffff) as u16,
        slot1,
        slot2,
    }
}

const _: () = {
    const OPCODE: u8 = 6;
    
    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op1))
    }
};

const _: () = {
    const OPCODE: u8 = 38;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op2))
    }
};

const _: () = {
    const OPCODE: u8 = 62;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op3))
    }
};

const _: () = {
    const OPCODE: u8 = 46;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op4))
    }
};

const _: () = {
    const OPCODE: u8 = 64;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op5))
    }
};

const _: () = {
    const OPCODE: u8 = 8;
    
    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op6))
    }
};

const _: () = {
    const OPCODE: u8 = 28;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op7))
    }
};

const _: () = {
    const OPCODE: u8 = 70;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op8))
    }
};

const _: () = {
    const OPCODE: u8 = 66;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op9))
    }
};

const _: () = {
    const OPCODE: u8 = 20;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(decode_ifbutton(&mut p, Op::Op10))
    }
};
