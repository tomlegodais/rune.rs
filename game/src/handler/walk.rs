use super::MessageHandler;
use crate::player::Player;
use crate::with_movement;
use crate::world::Position;
use macros::message_handler;
use net::WalkRequest;

#[message_handler]
async fn handle(player: &mut Player, msg: WalkRequest) {
    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, dest, msg.force_run)
        .await);
}
