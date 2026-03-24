use super::MessageHandler;
use crate::movement_ctx;
use crate::player::Player;
use crate::world::Position;
use macros::message_handler;
use net::inbound::walk::WalkRequest;

#[message_handler]
async fn handle(player: &mut Player, msg: WalkRequest) {
    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    player
        .movement
        .walk_to(movement_ctx!(player), dest, msg.force_run)
        .await;
}
