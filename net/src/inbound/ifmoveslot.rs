use macros::message_decoder;
use tokio_util::bytes::{Buf, Bytes};
use util::BufExt;

use super::{InboundDecoder, IncomingMessage};

pub struct IfMoveSlot {
    pub from_interface: u16,
    pub from_component: u16,
    pub from_slot: u16,
    pub to_interface: u16,
    pub to_component: u16,
    pub to_slot: u16,
}

const _: () = {
    const OPCODE: u8 = 75;

    #[message_decoder]
    fn decode_if_move_slot(mut payload: Bytes) -> IncomingMessage {
        let from_hash = payload.get_u32();
        let from_slot = payload.get_u16_le_add();
        let to_hash = payload.get_u32_mid_le();
        let to_slot = payload.get_u16_le_add();
        let _ = payload.get_u16_le();
        let _ = payload.get_u16_le();

        Box::new(IfMoveSlot {
            from_interface: (from_hash >> 16) as u16,
            from_component: (from_hash & 0xffff) as u16,
            from_slot,
            to_interface: (to_hash >> 16) as u16,
            to_component: (to_hash & 0xffff) as u16,
            to_slot,
        })
    }
};
