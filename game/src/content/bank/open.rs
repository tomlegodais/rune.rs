use net::{if_events, if_set_events};

use super::{BANK_COMPONENT, BANK_SIZE_VARC, FREE_BANK_SIZE_VARC, INV_COMPONENT, LAST_X_VARP};
use crate::player::{BANK_SIZE, Clientbound, InterfaceSlot, Player, banking as ui};

pub(super) async fn open(player: &mut Player) {
    player.interface_mut().open_slot(InterfaceSlot::Modal, ui::MAIN).await;
    player
        .interface_mut()
        .open_slot(InterfaceSlot::Inventory, ui::INV)
        .await;

    player.bank_mut().send_current_tab().await;

    player
        .if_set_events(if_set_events!(
            interface_id: ui::MAIN,
            component_id: BANK_COMPONENT,
            slots: [0 => BANK_SIZE as u16],
            right_click[0, 1, 2, 3, 4, 5, 6, 9],
            depth[2],
            can_drag_onto
        ))
        .await;

    player
        .if_set_events(if_set_events!(
            interface_id: ui::INV,
            component_id: INV_COMPONENT,
            slots: [0 => 27],
            right_click[0, 1, 2, 3, 4, 5, 9],
            use_on[components],
            depth[1],
            can_drag_onto
        ))
        .await;

    player.bank_mut().flush().await;
    let last_x = player.bank().last_x() as i32;
    player.varp_mut().send_varp(LAST_X_VARP, last_x).await;
    send_bank_size(player).await;
}

pub(super) async fn send_bank_size(player: &mut Player) {
    let total = (0..BANK_SIZE).filter_map(|i| player.bank().slot(i)).count() as i32;
    let free = (0..BANK_SIZE)
        .filter_map(|i| player.bank().slot(i))
        .filter(|obj| !crate::provider::get_obj_type(obj.id as u32).is_some_and(|t| t.members))
        .count() as i32;
    player.varp_mut().send_varc(BANK_SIZE_VARC, total).await;
    player.varp_mut().send_varc(FREE_BANK_SIZE_VARC, free).await;
}
