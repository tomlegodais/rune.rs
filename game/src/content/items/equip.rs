use filesystem::definition::{EquipmentFlag, EquipmentSlot};
use num_enum::TryFromPrimitive;

#[macros::on_obj(option = Two)]
async fn equip_item() {
    let Some(obj) = slot_obj!() else { return };
    let Some(def) = obj_def!(obj.id) else { return };
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
        && player
            .equipment()
            .slot(EquipmentSlot::Weapon)
            .is_some_and(|wep| displaced.contains(&wep))
    {
        unequip!(EquipmentSlot::Weapon);
    }

    player.equipment_mut().set(target_slot, Some(obj));

    for d in &displaced {
        player.inventory_mut().add(d.id, d.amount).await;
    }

    player.equipment_mut().flush().await;
    player.flush_appearance();
}

#[macros::on_interface(option = One, interface = 387)]
async fn unequip_item() {
    let slot = match component {
        8 => EquipmentSlot::Head,
        11 => EquipmentSlot::Cape,
        14 => EquipmentSlot::Amulet,
        17 => EquipmentSlot::Weapon,
        20 => EquipmentSlot::Body,
        23 => EquipmentSlot::Shield,
        26 => EquipmentSlot::Legs,
        29 => EquipmentSlot::Gloves,
        32 => EquipmentSlot::Boots,
        35 => EquipmentSlot::Ring,
        38 => EquipmentSlot::Ammo,
        _ => return,
    };

    let Some(obj) = player.equipment().slot(slot) else { return };

    if player.inventory().free_slots() == 0 {
        send_message!("Not enough inventory space.");
        return;
    }

    player.equipment_mut().set(slot, None);
    player.inventory_mut().add(obj.id, obj.amount).await;
    player.equipment_mut().flush().await;
    player.flush_appearance();
}
