use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};

use super::{InboundDecoder, IncomingMessage};

pub struct ExamLoc {
    pub id: u16,
}

const _: () = {
    const OPCODE: u8 = 73;

    #[message_decoder]
    fn decode(mut payload: Bytes) -> IncomingMessage {
        Box::new(ExamLoc { id: payload.get_u16() })
    }
};
