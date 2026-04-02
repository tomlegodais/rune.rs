use macros::command;

use crate::{
    command::CommandEntry,
    player::{Clientbound, Player, Stat},
};

#[command(name = "test")]
async fn test(player: &mut Player, stat_id: usize) {
    let Ok(stat) = Stat::try_from(stat_id) else {
        player.send_message("Invalid stat id (0-23)").await;
        return;
    };

    player.stat_mut().add_xp(stat, 50_000.0).await;
}
