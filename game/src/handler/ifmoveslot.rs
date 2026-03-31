use macros::message_handler;
use net::IfMoveSlot;

use super::MessageHandler;
use crate::player::{INV_SIZE, Player};

#[message_handler]
async fn handle_if_move_slot(player: &mut Player, msg: IfMoveSlot) {
    if msg.from_interface != 149 || msg.from_component != 0 || msg.to_interface != 149 || msg.to_component != 0 {
        return;
    }

    let from = msg.from_slot as usize;
    let to = msg.to_slot as usize - INV_SIZE;
    if from >= INV_SIZE || to >= INV_SIZE {
        return;
    }

    player.inv_mut().swap(from, to).await;
}
