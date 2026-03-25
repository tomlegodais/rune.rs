use super::CommandEntry;
use crate::player::{Player, VarpManager};
use crate::{provider, send_message};
use macros::command;

#[command(name = "varbit")]
async fn inspect(player: &mut Player, id: u32) {
    match provider::get_varbit_definition(id) {
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
    {
        let mut varps = player.systems.guard::<VarpManager>();
        varps.send_varbit(id, value).await;
    }
    send_message!(player, "varbit {} = {}", id, value);
}
