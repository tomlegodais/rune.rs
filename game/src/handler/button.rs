use super::MessageHandler;
use crate::player::{Movement, Player};
use crate::{send_message, with_movement};
use macros::message_handler;
use net::ButtonClick;

#[message_handler]
async fn handle(player: &mut Player, msg: ButtonClick) {
    match (msg.opcode, msg.interface, msg.component) {
        (6, 750, 1) | (6, 261, 3) => {
            let running = !player.system::<Movement>().running;
            with_movement!(player, |m, ctx| m.set_run(&mut ctx, running).await);
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
