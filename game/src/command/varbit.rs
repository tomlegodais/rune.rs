use super::CommandEntry;
use crate::player::Player;
use crate::send_message;
use crate::world::Varbits;
use macros::command;

#[command(name = "varbit")]
async fn inspect(player: &mut Player, id: u32) {
    match Varbits::get(id) {
        Some(def) => send_message!(
            player,
            "varbit {}: varp={}, bits={}-{}, mask={}",
            id,
            def.varp,
            def.low_bit,
            def.high_bit,
            def.mask()
        ),
        None => send_message!(player, "varbit {} not found", id),
    }
}

#[command(name = "setvarbit")]
async fn set(player: &mut Player, id: u32, value: i32) {
    player.varps.send_varbit(id, value).await;
    send_message!(player, "varbit {} = {}", id, value);
}
