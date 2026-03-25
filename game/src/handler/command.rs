use super::MessageHandler;
use crate::player::Player;
use macros::message_handler;
use net::ClientCommand;

#[message_handler]
async fn handle(player: &mut Player, msg: ClientCommand) {
    let mut parts = msg.command.splitn(2, ' ');
    let name = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    crate::command::dispatch(player, msg.client_sent, name, args).await;
}
