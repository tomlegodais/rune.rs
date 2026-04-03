use macros::message_handler;
use net::IfSubClosed;

use super::MessageHandler;
use crate::player::Player;

#[message_handler]
async fn handle_if_sub_closed(player: &mut Player, _msg: IfSubClosed) {
    player.cancel_action(true).await;
}
