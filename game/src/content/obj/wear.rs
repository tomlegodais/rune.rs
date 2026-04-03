use filesystem::config::{WearFlag, WearPos};
use net::{ScriptArg, if_events, if_set_events};
use num_enum::TryFromPrimitive;

use crate::player::{Clientbound, InterfaceSlot, Obj, Player, equipment};

#[macros::on_obj(op = Op2)]
async fn wear_obj() {
    let Some(obj) = slot_obj!() else { return };
    wear_slot(&mut player, slot as usize, obj).await;
}

#[macros::on_interface(op = 1, interface = 387)]
async fn unwear_obj() {
    match component {
        8 => unwear_slot(&mut player, WearPos::Head).await,
        11 => unwear_slot(&mut player, WearPos::Cape).await,
        14 => unwear_slot(&mut player, WearPos::Amulet).await,
        17 => unwear_slot(&mut player, WearPos::Weapon).await,
        20 => unwear_slot(&mut player, WearPos::Body).await,
        23 => unwear_slot(&mut player, WearPos::Shield).await,
        26 => unwear_slot(&mut player, WearPos::Legs).await,
        29 => unwear_slot(&mut player, WearPos::Gloves).await,
        32 => unwear_slot(&mut player, WearPos::Boots).await,
        35 => unwear_slot(&mut player, WearPos::Ring).await,
        38 => unwear_slot(&mut player, WearPos::Ammo).await,
        39 => open_equipment_stats(&mut player).await,
        _ => {}
    }
}

#[macros::on_interface(op = 1, interface = 670, component = 0)]
async fn equip_from_stats_screen() {
    let Some(obj) = player.inv().slot(slot1 as usize) else { return };
    wear_slot(&mut player, slot1 as usize, obj).await;
}

#[macros::on_interface(op = 1, interface = 667, component = 14)]
async fn unequip_from_stats_screen() {
    let Ok(wear_pos) = WearPos::try_from_primitive(slot1 as usize) else { return };
    unwear_slot(&mut player, wear_pos).await;
}

async fn wear_slot(player: &mut Player, inv_slot: usize, obj: Obj) {
    let Some(obj_type) = crate::provider::get_obj_type(obj.id as u32) else { return };
    let Some(raw_slot) = obj_type.wearpos else { return };
    let Ok(target_slot) = WearPos::try_from_primitive(raw_slot as usize) else { return };

    let displaced = player.worn().displace(target_slot, obj_type.wearflag);
    let needed = displaced.len().saturating_sub(1);
    if player.inv().free_slots() < needed {
        crate::send_message!(player, "Not enough inventory space.");
        return;
    }

    player.inv_mut().clear_slot(inv_slot).await;
    if obj_type.wearflag == WearFlag::TwoHanded {
        player.worn_mut().set(WearPos::Shield, None);
    }

    if target_slot == WearPos::Shield
        && player
            .worn()
            .slot(WearPos::Weapon)
            .is_some_and(|wep| displaced.contains(&wep))
    {
        player.worn_mut().set(WearPos::Weapon, None);
    }

    player.worn_mut().set(target_slot, Some(obj));

    for d in &displaced {
        player.inv_mut().add(d.id, d.amount).await;
    }

    player.worn_mut().flush().await;
    player.appearance_mut().flush();
}

async fn unwear_slot(player: &mut Player, slot: WearPos) {
    let Some(obj) = player.worn().slot(slot) else { return };
    if player.inv().free_slots() == 0 {
        crate::send_message!(player, "Not enough inventory space.");
        return;
    }
    player.worn_mut().set(slot, None);
    player.inv_mut().add(obj.id, obj.amount).await;
    player.worn_mut().flush().await;
    player.appearance_mut().flush();
}

async fn open_equipment_stats(player: &mut Player) {
    player
        .interface_mut()
        .open_slot(InterfaceSlot::Inventory, equipment::INV)
        .await;

    player
        .interface_mut()
        .open_slot(InterfaceSlot::Modal, equipment::STATS)
        .await;

    player.run_client_script(787, vec![ScriptArg::Int(1)]).await;
    player.inv_mut().flush().await;
    player
        .set_items_options(equipment::INV, 0, 93, 4, 7, &["Wear", "Examine"])
        .await;

    player
        .if_set_events(if_set_events!(
            interface_id: equipment::INV,
            component_id: 0,
            slots: [0 => 27],
            right_click[0, 1, 2, 3]
        ))
        .await;

    player
        .if_set_events(if_set_events!(
            interface_id: equipment::STATS,
            component_id: 14,
            slots: [0 => 13],
            right_click[0, 9]
        ))
        .await;
}
