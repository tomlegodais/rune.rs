use filesystem::definition::{EquipmentFlag, EquipmentSlot};
use num_enum::TryFromPrimitive;

use crate::{player::Player, send_message};

#[macros::on_item(option = Two)]
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
        && player
            .equipment()
            .slot(EquipmentSlot::Weapon)
            .is_some_and(|wep| displaced.contains(&wep))
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

pub async fn unequip_item(player: &mut Player, component: u16) {
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

    let Some(item) = player.equipment().slot(slot) else { return };

    if player.inventory().free_slots() == 0 {
        send_message!(player, "Not enough inventory space.");
        return;
    }

    player.equipment_mut().set(slot, None);
    player.inventory_mut().add(item.id, item.amount).await;
    player.equipment_mut().flush().await;
    player.flush_appearance();
}
