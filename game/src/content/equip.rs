use filesystem::definition::{EquipmentFlag, EquipmentSlot};
use num_enum::TryFromPrimitive;

#[macros::on_item_option(option = 2)]
async fn equip_item(player: &mut Player, slot: u16) {
    let Some(item) = slot_item!() else { return };
    let Some(def) = item_def!(item.id) else { return };
    let Some(raw_slot) = def.equipment_slot else { return };
    let Ok(target_slot) = EquipmentSlot::try_from_primitive(raw_slot as usize) else { return };

    let displaced = player.equipment().displace(target_slot, def.equipment_flag);
    let needed = displaced.len().saturating_sub(1);
    if player.inventory().free_slots() < needed {
        send_message!("Not enough inventory space.");
        return;
    }

    player.inventory_mut().clear_slot(slot as usize).await;
    if def.equipment_flag == EquipmentFlag::TwoHanded {
        unequip!(EquipmentSlot::Shield);
    }
    if target_slot == EquipmentSlot::Shield
        && player.equipment().slot(EquipmentSlot::Weapon).is_some_and(|wep| displaced.contains(&wep))
    {
        unequip!(EquipmentSlot::Weapon);
    }
    player.equipment_mut().set(target_slot, Some(item));

    for d in &displaced {
        player.inventory_mut().add(d.id, d.amount).await;
    }

    player.equipment_mut().flush().await;
    player.flush_appearance();
}
