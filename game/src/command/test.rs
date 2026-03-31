use macros::command;

use crate::{
    command::CommandEntry,
    player::{Player, chatbox},
};

#[command(name = "test")]
async fn test(player: &mut Player) {
    player.interface_mut().open_sub(&chatbox::LEVEL_UP).await;

    player
        .interface_mut()
        .set_text(
            &chatbox::LEVEL_UP,
            0,
            "Congratulations, you have just advanced an Attack level!",
        )
        .await;

    player
        .interface_mut()
        .set_text(&chatbox::LEVEL_UP, 1, "You have now reached level 77.")
        .await;

    player.varp_mut().send_varbit(4757, 1).await;
}
