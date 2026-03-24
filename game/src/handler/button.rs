use super::MessageHandler;
use crate::movement_ctx;
use crate::player::Player;
use macros::message_handler;
use net::inbound::button::ButtonClick;

#[message_handler]
async fn handle(player: &mut Player, msg: ButtonClick) {
    match (msg.opcode, msg.interface, msg.component) {
        (6, 750, 1) => {
            let running = !player.movement.running;
            player
                .movement
                .set_run(movement_ctx!(player), running)
                .await;
        }
        _ => {}
    }
}
