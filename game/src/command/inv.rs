use super::CommandEntry;
use crate::player::{Inventory, Player};
use crate::send_message;
use macros::command;

#[command(name = "inv_add")]
async fn add(player: &mut Player, item_id: u16, amount: u32) {
    let leftover = {
        let mut inv = player.systems.guard::<Inventory>();
        inv.add(item_id, amount).await
    };
    send_message!(player, "Added {}x item {}.", amount - leftover, item_id);
}

#[command(name = "inv_remove")]
async fn remove(player: &mut Player, item_id: u16, amount: u32) {
    let leftover = {
        let mut inv = player.systems.guard::<Inventory>();
        inv.remove(item_id, amount).await
    };
    send_message!(player, "Removed {}x item {}.", amount - leftover, item_id);
}

#[command(name = "inv_clear")]
async fn clear(player: &mut Player) {
    {
        let mut inv = player.systems.guard::<Inventory>();
        inv.clear().await;
    }
    send_message!(player, "Inventory cleared.");
}
