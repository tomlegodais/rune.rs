use macros::command;

use super::{CommandEntry, RawArgs};
use crate::{player::Player, send_message, world::Position};

#[command(name = "tele")]
async fn handle(player: &mut Player, client_sent: bool, args: RawArgs) {
    let (x, y, plane) = if client_sent {
        let parts: Vec<i32> = args.0.split(',').filter_map(|s| s.parse().ok()).collect();
        if parts.len() < 5 {
            send_message!(player, "Usage: ::tele <x> <y> [plane]");
            return;
        }
        ((parts[1] << 6) | parts[3], (parts[2] << 6) | parts[4], parts[0])
    } else {
        let parts: Vec<i32> = args.0.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        if parts.len() < 2 {
            send_message!(player, "Usage: ::tele <x> <y> [plane]");
            return;
        }
        (parts[0], parts[1], parts.get(2).copied().unwrap_or(0))
    };

    let dest = Position::new(x, y, plane);
    player.movement_mut().teleport(dest).await;
    send_message!(player, "Teleporting to {}, {}, {}", x, y, plane);
}
