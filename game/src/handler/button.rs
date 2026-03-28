use super::MessageHandler;
use crate::player::Player;
use crate::{send_message, with_movement};
use macros::message_handler;
use net::{ButtonClick, ClickOption};

#[message_handler]
async fn handle(player: &mut Player, msg: ButtonClick) {
    match (msg.opcode, msg.interface, msg.component) {
        (6, 750, 1) | (6, 261, 3) => {
            let running = player.movement().running;
            with_movement!(player, |m, ctx| m.set_run(&mut ctx, running).await);
        }
        (6, 182, 6) => player.logout().await,
        (6, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::One),
        (38, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Two),
        (62, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Three),
        (46, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Four),
        (46, 64, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Five),
        (8, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Six),
        (28, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Seven),
        (70, 149, 0) => crate::handler::dispatch_item(player, msg.slot1, ClickOption::Eight),
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
