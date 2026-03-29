use macros::message_handler;
use net::PublicChat;
use util::format_sentence;

use super::MessageHandler;
use crate::{
    player::{ChatMask, Player},
    provider,
};

#[message_handler]
async fn handle(player: &mut Player, msg: PublicChat) {
    let message = provider::decode_huffman(&msg.payload, msg.text_len);

    player.player_info.add_mask(ChatMask {
        message: format_sentence(&message),
        color: msg.color,
        effect: msg.effect,
        rights: player.rights,
    });
}
