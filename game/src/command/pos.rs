use macros::command;

use super::CommandEntry;
use crate::{player::Player, send_message};

#[command(name = "pos")]
async fn handle(player: &mut Player) {
    let pos = player.position;
    send_message!(player, "Position: x={}, y={}, plane={}", pos.x, pos.y, pos.plane);
}
