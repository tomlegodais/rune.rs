use super::MessageHandler;
use crate::player::Player;
use crate::{movement_ctx, send_message};
use macros::message_handler;
use net::inbound::button::ButtonClick;

#[message_handler]
async fn handle(player: &mut Player, msg: ButtonClick) {
    match (msg.opcode, msg.interface, msg.component) {
        (6, 750, 1) | (6, 261, 3) => {
            let running = !player.movement.running;
            player
                .movement
                .set_run(movement_ctx!(player), running)
                .await;
        }
        (6, 182, 6) => player.logout().await,
        _ => {
            send_message!(
                player,
                "Unhandled Button (opcode={}, interface={}, component={})",
                msg.opcode,
                msg.interface,
                msg.component
            );
        }
    }
}
