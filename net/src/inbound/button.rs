use super::{InboundDecoder, IncomingMessage};
use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

pub struct ButtonClick {
    pub opcode: u8,
    pub interface: u16,
    pub component: u16,
    pub slot1: u16,
    pub slot2: u16,
}

macro_rules! button_decoder {
    ($opcode:literal, $fn_name:ident) => {
        const _: () = {
            const OPCODE: u8 = $opcode;

            #[message_decoder]
            fn $fn_name(mut payload: Bytes) -> IncomingMessage {
                let hash = payload.get_u32();
                let slot1 = payload.get_u16_le();
                let slot2 = payload.get_u16_le();
                let interface = (hash >> 16) as u16;
                let component = (hash & 0xffff) as u16;
                Box::new(ButtonClick {
                    opcode: $opcode,
                    interface,
                    component,
                    slot1,
                    slot2,
                })
            }
        };
    };
}

button_decoder!(6, decode_6);
button_decoder!(38, decode_38);
button_decoder!(62, decode_62);
button_decoder!(46, decode_46);
button_decoder!(64, decode_64);
button_decoder!(8, decode_8);
button_decoder!(28, decode_28);
button_decoder!(70, decode_70);
button_decoder!(66, decode_66);
button_decoder!(20, decode_20);
