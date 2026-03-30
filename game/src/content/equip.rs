use filesystem::definition::EquipmentFlag;
use num_enum::TryFromPrimitive;

use crate::player::EquipmentSlot;

#[macros::on_item_option(option = 2)]
async fn equip_item(player: &mut Player, slot: u16) {
    let Some(item) = slot_item!() else {
        return;
    };
    let item_id = item.id;
    let amount = item.amount;

    let Some(def) = crate::provider::get_item_definition(item_id as u32) else {
        return;
    };

    let Some(raw_slot) = def.equipment_slot else {
        return;
    };

    let Ok(target_slot) = EquipmentSlot::try_from_primitive(raw_slot as usize) else {
        return;
    };

    let two_handed = def.equipment_flag == EquipmentFlag::TwoHanded;

    let mut returning: Vec<(u16, u32)> = Vec::new();

    if let Some(item) = player.equipment().slot(target_slot) {
        returning.push(item);
    }

    if two_handed && let Some(shield) = player.equipment().slot(EquipmentSlot::Shield) {
        returning.push(shield);
    }

    if target_slot == EquipmentSlot::Shield
        && let Some((wep_id, wep_amt)) = player.equipment().slot(EquipmentSlot::Weapon)
        && crate::provider::get_item_definition(wep_id as u32)
            .is_some_and(|d| d.equipment_flag == EquipmentFlag::TwoHanded)
    {
        returning.push((wep_id, wep_amt));
    }

    let needed = returning.len().saturating_sub(1);
    if player.inventory().free_slots() < needed {
        send_message!("Not enough inventory space.");
        return;
    }

    player.inventory_mut().clear_slot(slot as usize).await;

    if two_handed {
        player.equipment_mut().set(EquipmentSlot::Shield, None);
    }
    if target_slot == EquipmentSlot::Shield
        && returning.iter().any(|&(id, _)| {
            crate::provider::get_item_definition(id as u32)
                .is_some_and(|d| d.equipment_flag == EquipmentFlag::TwoHanded)
        })
    {
        player.equipment_mut().set(EquipmentSlot::Weapon, None);
    }
    player.equipment_mut().set(target_slot, Some((item_id, amount)));

    for (ret_id, ret_amt) in &returning {
        player.inventory_mut().add(*ret_id, *ret_amt).await;
    }

    player.equipment_mut().flush().await;

    player.flush_appearance();
}
