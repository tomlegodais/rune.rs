use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};

#[derive(Debug)]
pub struct IfDialogContinue {
    pub interface_id: u16,
    pub component_id: u16,
    pub button_id: u8,
}

const _: () = {
    const OPCODE: u8 = 18;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        let _unused = p.get_u16_add();
        let hash = p.get_u32();
        let interface_id = (hash >> 16) as u16;
        let component_id = (hash & 0xffff) as u16;
        let button_id = (hash & 0xff) as u8;

        Box::new(IfDialogContinue {
            interface_id,
            component_id,
            button_id,
        })
    }
};
