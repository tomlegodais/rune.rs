use macros::message_handler;
use net::IfDialogContinue;

use super::MessageHandler;
use crate::player::Player;

#[message_handler]
async fn handle_dialogue_continue(player: &mut Player, msg: IfDialogContinue) {
    let is_options = msg.interface_id > crate::player::OPTIONS_BASE;
    let choice = is_options.then_some(msg.component_id as u8).unwrap_or(0);

    player.dialogue_mut().respond(choice).await;
}
