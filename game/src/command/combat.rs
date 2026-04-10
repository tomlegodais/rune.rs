use macros::command;

use super::CommandEntry;
use crate::{
    content::combat::{formula, player as combat_player},
    player::Player,
    send_message,
};

#[command(name = "maxhit")]
async fn maxhit(player: &mut Player) {
    let (atk, style) = combat_player::melee_atk(player);
    let max = formula::max_hit(&atk);
    send_message!(
        player,
        "Max hit: {} (style: {:?}, atk bonus: {}, str bonus: {})",
        max,
        style.atk_type,
        atk.atk_bonus,
        atk.str_bonus
    );
}
