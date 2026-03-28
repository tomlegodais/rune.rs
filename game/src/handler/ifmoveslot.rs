use super::MessageHandler;
use crate::player::Player;
use macros::message_handler;
use net::IfMoveSlot;

#[message_handler]
async fn handle_if_move_slot(player: &mut Player, msg: IfMoveSlot) {
    if msg.from_interface != 149
        || msg.from_component != 0
        || msg.to_interface != 149
        || msg.to_component != 0
    {
        return;
    }

    let from = msg.from_slot as usize;
    let to = msg.to_slot as usize;
    if from >= 28 || to >= 28 {
        return;
    }

    player.inventory_mut().swap(from, to).await;
}
