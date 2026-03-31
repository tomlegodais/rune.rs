use macros::command;

use crate::{command::CommandEntry, player::Player, send_message};

#[command(name = "spawnloc")]
async fn spawnloc(player: &mut Player, id: u16, loc_type: Option<u8>, rotation: Option<u8>, ticks: Option<u16>) {
    let loc_type = loc_type.unwrap_or(10);
    let rotation = rotation.unwrap_or(0);
    let ticks = ticks.unwrap_or(100);
    let pos = player.position;
    let world = player.world();
    world.locs.spawn(pos, id, loc_type, rotation, ticks);
    send_message!(
        player,
        "Spawned loc {} at {},{},{} for {} ticks",
        id,
        pos.x,
        pos.y,
        pos.plane,
        ticks
    );
}
