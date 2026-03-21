use super::CommandEntry;
use crate::player::{Player, Skill};
use crate::send_message;
use macros::command;

#[command(name = "setlevel")]
async fn handle(player: &mut Player, skill_id: usize, level: u8) {
    let skill = match Skill::try_from(skill_id) {
        Ok(s) => s,
        Err(_) => {
            player.send_message("Invalid skill id (0-23)").await;
            return;
        }
    };

    if !(1..=99).contains(&level) {
        player.send_message("Invalid level (1-99)").await;
        return;
    }

    player.skills.set_level(skill, level);
    player.skills.flush().await;
    send_message!(player, "Set {:?} to level {}", skill, level);
}
