use macros::command;
use num_enum::TryFromPrimitive;
use rand::Rng;

use super::CommandEntry;
use crate::{
    player::{Clientbound, NUM_STATS, Player, Stat},
    send_message,
};

#[command(name = "addxp")]
async fn add_xp(player: &mut Player, stat_id: usize, xp: f64) {
    if let Ok(stat) = Stat::try_from_primitive(stat_id) {
        player.stat_mut().add_xp(stat, xp).await;
        player.send_message(format!("Added {} xp to {:?}.", xp, stat)).await;
        return;
    }

    player.send_message("Invalid stat id (0-23)").await;
}

#[command(name = "setlevel")]
async fn setlevel(player: &mut Player, stat_id: usize, level: u8) {
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
    player.appearance_mut().flush();

    send_message!(player, "Set {:?} to level {}", stat, level);
}

#[command(name = "random_stats")]
async fn random_stats(player: &mut Player) {
    let levels: Vec<_> = {
        let mut rng = rand::rng();
        (0..NUM_STATS)
            .filter_map(|i| Stat::try_from(i).ok())
            .map(|stat| (stat, rng.random_range(92u8..=99)))
            .collect()
    };

    for (stat, level) in levels {
        player.stat_mut().set_level(stat, level);
    }

    player.appearance_mut().flush();
    player.stat_mut().flush().await;
}
