use macros::message_handler;
use net::MoveClick;

use super::MessageHandler;
use crate::{
    player::{Player, is_action_locked},
    world::Position,
};

#[message_handler]
async fn handle(player: &mut Player, msg: MoveClick) {
    if is_action_locked(player) {
        return;
    }

    player.cancel_action().await;
    player.interaction_mut().clear();

    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    player.movement_mut().walk_to(dest, msg.ctrl_run, None).await;
}
