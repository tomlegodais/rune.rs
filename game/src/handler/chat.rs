use super::MessageHandler;
use crate::player::{ChatMask, Player};
use macros::message_handler;
use net::inbound::chat::PublicChat;

#[message_handler]
async fn handle(player: &mut Player, msg: PublicChat) {
    player.player_info.add_mask(ChatMask {
        message: msg.message,
        color: msg.color,
        effect: msg.effect,
        rights: player.rights,
    });
}
