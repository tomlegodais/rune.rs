use macros::command;

use super::CommandEntry;
use crate::{
    player::{Clientbound, Player, Stat},
    send_message,
};

#[command(name = "setlevel")]
async fn handle(player: &mut Player, stat_id: usize, level: u8) {
    let stat = match Stat::try_from(stat_id) {
        Ok(s) => s,
        Err(_) => {
            player.send_message("Invalid stat id (0-23)").await;
            return;
        }
    };

    if !(1..=99).contains(&level) {
        player.send_message("Invalid level (1-99)").await;
        return;
    }

    player.stat_mut().set_level(stat, level);
    player.stat_mut().flush().await;

    send_message!(player, "Set {:?} to level {}", stat, level);
}
