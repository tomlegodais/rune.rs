use crate::command::CommandEntry;
use crate::player::Player;
use macros::command;

#[command(name = "test")]
async fn set(player: &mut Player) {
    player.send_message("This is a test command").await;
}
