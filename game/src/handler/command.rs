use macros::message_handler;
use net::ClientCheat;

use super::MessageHandler;
use crate::{command::dispatch_command, player::Player};

#[message_handler]
async fn handle(player: &mut Player, msg: ClientCheat) {
    let mut parts = msg.command.splitn(2, ' ');
    let name = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    dispatch_command(player, msg.client_sent, name, args).await;
}
