use filesystem::definition::EquipmentFlag;
use num_enum::TryFromPrimitive;

use crate::player::{EquipmentSlot, Item};

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

    let returning = player.equipment().displace(target_slot, def.equipment_flag);

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
        && returning.iter().any(|item| {
            crate::provider::get_item_definition(item.id as u32)
                .is_some_and(|d| d.equipment_flag == EquipmentFlag::TwoHanded)
        })
    {
        player.equipment_mut().set(EquipmentSlot::Weapon, None);
    }
    player.equipment_mut().set(target_slot, Some(Item::new(item_id, amount)));

    for item in &returning {
        player.inventory_mut().add(item.id, item.amount).await;
    }

    player.equipment_mut().flush().await;

    player.flush_appearance();
}
