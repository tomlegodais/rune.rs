use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

use super::{ClickOption, InboundDecoder, IncomingMessage};

pub struct ButtonClick {
    pub option: ClickOption,
    pub interface: u16,
    pub component: u16,
    pub slot1: u16,
    pub slot2: u16,
}

macro_rules! button_decoder {
    ($opcode:literal, $fn_name:ident, $option:expr) => {
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
                    option: $option,
                    interface,
                    component,
                    slot1,
                    slot2,
                })
            }
        };
    };
}

button_decoder!(6, decode_6, ClickOption::One);
button_decoder!(38, decode_38, ClickOption::Two);
button_decoder!(62, decode_62, ClickOption::Three);
button_decoder!(46, decode_46, ClickOption::Four);
button_decoder!(64, decode_64, ClickOption::Five);
button_decoder!(8, decode_8, ClickOption::Six);
button_decoder!(28, decode_28, ClickOption::Seven);
button_decoder!(70, decode_70, ClickOption::Eight);
button_decoder!(66, decode_66, ClickOption::Nine);
button_decoder!(20, decode_20, ClickOption::Ten);
