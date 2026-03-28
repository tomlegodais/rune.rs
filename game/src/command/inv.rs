use super::CommandEntry;
use crate::player::Player;
use crate::send_message;
use macros::command;

#[command(name = "item")]
async fn add(player: &mut Player, item_id: u16, amount: Option<u32>) {
    let amount = amount.unwrap_or(1);
    let leftover = player.inventory_mut().add(item_id, amount).await;
    send_message!(player, "Added {}x item {}.", amount - leftover, item_id);
}

#[command(name = "clear")]
async fn clear(player: &mut Player) {
    player.inventory_mut().clear().await;
    send_message!(player, "Inventory cleared.");
}
