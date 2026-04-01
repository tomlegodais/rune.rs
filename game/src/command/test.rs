use macros::command;

use crate::{
    chatbox_dialogue,
    command::CommandEntry,
    player::{Player, chatbox},
};

#[command(name = "test")]
async fn test(player: &mut Player) {
    chatbox_dialogue!(
        player,
        &chatbox::LEVEL_UP,
        "Congratulations, you have just advanced an Attack level!",
        "You have now reached level 77.",
    );

    player.varp_mut().send_varbit(4757, 1).await;
}
