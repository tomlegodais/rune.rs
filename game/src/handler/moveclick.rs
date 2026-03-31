use macros::message_handler;
use net::MoveClick;

use super::MessageHandler;
use crate::{
    player::{Player, is_action_locked},
    with_movement,
    world::Position,
};

#[message_handler]
async fn handle(player: &mut Player, msg: MoveClick) {
    if is_action_locked(player) {
        return;
    }

    player.world().action_states.lock().remove(&player.index);
    player.interaction_mut().clear();

    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    with_movement!(player, |m, ctx| m.walk_to(&mut ctx, dest, msg.ctrl_run, None).await);
}
