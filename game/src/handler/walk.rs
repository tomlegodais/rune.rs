use super::MessageHandler;
use crate::player::{Interaction, Player, is_action_locked};
use crate::with_movement;
use crate::world::Position;
use macros::message_handler;
use net::WalkRequest;

#[message_handler]
async fn handle(player: &mut Player, msg: WalkRequest) {
    if is_action_locked(player) {
        return;
    }

    player.world().action_states.lock().remove(&player.index);
    player.systems.get_mut::<Interaction>().clear();
    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, dest, msg.force_run, false)
        .await);
}