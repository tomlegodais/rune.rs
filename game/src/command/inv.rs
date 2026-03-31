use macros::command;

use super::CommandEntry;
use crate::{player::Player, send_message};

#[command(name = "inv_add")]
async fn add(player: &mut Player, obj_id: u16, amount: Option<u32>) {
    let amount = amount.unwrap_or(1);
    let leftover = player.inv_mut().add(obj_id, amount).await;
    send_message!(player, "Added {}x obj {}.", amount - leftover, obj_id);
}

#[command(name = "inv_clear")]
async fn clear(player: &mut Player) {
    player.inv_mut().clear().await;
    send_message!(player, "Inventory cleared.");
}
