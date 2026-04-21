use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

use super::{InboundDecoder, IncomingMessage};

#[derive(Debug)]
pub struct ResumeCountDialog {
    pub value: u32,
}

const _: () = {
    const OPCODE: u8 = 81;

    #[message_decoder]
    fn decode(mut p: Bytes) -> IncomingMessage {
        Box::new(ResumeCountDialog { value: p.get_u32() })
    }
};
