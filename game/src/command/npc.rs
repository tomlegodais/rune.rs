use crate::command::CommandEntry;
use crate::player::Player;
use crate::send_message;
use macros::command;

#[command(name = "nftalk")]
async fn handle(player: &mut Player) {
    player
        .world()
        .npc_mut(1)
        .force_talk("This is a test message".to_string());

    send_message!(player, "NPC force talk sent");
}
