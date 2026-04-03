use macros::command;

use crate::{
    command::CommandEntry,
    player::{InterfaceSlot, Player},
};

#[command(name = "test")]
async fn test(player: &mut Player, id: usize) {
    player.interface_mut().open_slot(InterfaceSlot::Modal, id as u16).await;
}
