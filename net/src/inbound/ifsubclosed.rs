use macros::message_decoder;
use tokio_util::bytes::Bytes;

use super::{InboundDecoder, IncomingMessage};

pub struct IfSubClosed;

const _: () = {
    const OPCODE: u8 = 69;

    #[message_decoder]
    fn decode(_: Bytes) -> IncomingMessage {
        Box::new(IfSubClosed)
    }
};
