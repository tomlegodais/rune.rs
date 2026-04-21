use macros::message_handler;
use net::IfMoveSlot;

use super::MessageHandler;
use crate::player::{BANK_SIZE, INV_SIZE, Player};

#[message_handler]
async fn handle_if_move_slot(player: &mut Player, msg: IfMoveSlot) {
    let from = msg.from_slot as usize;
    let to = msg.to_slot as usize;

    match (msg.from_interface, msg.from_component) {
        (149, 0) if msg.to_interface == 149 && msg.to_component == 0 => {
            let to = to.saturating_sub(INV_SIZE);
            if from < INV_SIZE && to < INV_SIZE {
                player.inv_mut().swap(from, to).await;
            }
        }
        (763, 0) if msg.to_interface == 763 && msg.to_component == 0 && from < INV_SIZE && to < INV_SIZE => {
            player.inv_mut().swap(from, to).await;
        }
        (762, 87) if msg.to_interface == 762 && msg.to_component == 87 && from < BANK_SIZE && to < BANK_SIZE => {
            if player.bank().insert_mode() {
                player.bank_mut().insert(from, to).await;
            } else {
                player.bank_mut().swap(from, to).await;
            }
        }
        (762, 87) if msg.to_interface == 762 && msg.to_slot == 65535 => {
            if let Some(tab) = tab_from_component(msg.to_component)
                && from < BANK_SIZE
            {
                player.bank_mut().move_to_tab(from, tab).await;
            }
        }
        _ => {}
    }
}

fn tab_from_component(c: u16) -> Option<u8> {
    ((40..=56).contains(&c) && (c - 40).is_multiple_of(2)).then(|| 8 - ((c - 40) / 2) as u8)
}
