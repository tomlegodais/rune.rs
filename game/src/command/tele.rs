use super::CommandEntry;
use crate::player::Player;
use crate::send_message;
use crate::world::Position;
use macros::command;

#[command(name = "tele")]
async fn handle(player: &mut Player, x: i32, y: i32, plane: i32) {
    let dest = Position::new(x, y, plane);
    player.teleport(dest);
    send_message!(player, "Teleporting to {}, {}, {}", x, y, plane);
}
