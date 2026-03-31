use macros::command;

use super::CommandEntry;
use crate::{player::Player, send_message};

#[command(name = "inv_add")]
async fn add(player: &mut Player, item_id: u16, amount: Option<u32>) {
    let amount = amount.unwrap_or(1);
    let leftover = player.inv_mut().add(item_id, amount).await;
    send_message!(player, "Added {}x item {}.", amount - leftover, item_id);
}

#[command(name = "inv_clear")]
async fn clear(player: &mut Player) {
    player.inv_mut().clear().await;
    send_message!(player, "Inventory cleared.");
}
