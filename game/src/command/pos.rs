use super::CommandEntry;
use crate::player::Player;
use crate::send_message;
use macros::command;

#[command(name = "pos")]
async fn handle(player: &mut Player) {
    let pos = player.entity.position;
    send_message!(
        player,
        "Position: x={}, y={}, plane={}",
        pos.x,
        pos.y,
        pos.plane
    );
}
