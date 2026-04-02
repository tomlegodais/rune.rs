use filesystem::config::{WearFlag, WearPos};
use num_enum::TryFromPrimitive;

#[macros::on_obj(op = Op2)]
async fn wear_obj() {
    let Some(obj) = slot_obj!() else { return };
    let Some(obj_type) = obj_def!(obj.id) else { return };
    let Some(raw_slot) = obj_type.wearpos else { return };
    let Ok(target_slot) = WearPos::try_from_primitive(raw_slot as usize) else { return };

    let displaced = player.worn().displace(target_slot, obj_type.wearflag);
    let needed = displaced.len().saturating_sub(1);
    if player.inv().free_slots() < needed {
        send_message!("Not enough inventory space.");
        return;
    }

    player.inv_mut().clear_slot(slot as usize).await;
    if obj_type.wearflag == WearFlag::TwoHanded {
        unwear!(WearPos::Shield);
    }

    if target_slot == WearPos::Shield
        && player
            .worn()
            .slot(WearPos::Weapon)
            .is_some_and(|wep| displaced.contains(&wep))
    {
        unwear!(WearPos::Weapon);
    }

    player.worn_mut().set(target_slot, Some(obj));

    for d in &displaced {
        player.inv_mut().add(d.id, d.amount).await;
    }

    player.worn_mut().flush().await;
    player.appearance_mut().flush();
}

#[macros::on_interface(op = 1, interface = 387)]
async fn unwear_obj() {
    let slot = match component {
        8 => WearPos::Head,
        11 => WearPos::Cape,
        14 => WearPos::Amulet,
        17 => WearPos::Weapon,
        20 => WearPos::Body,
        23 => WearPos::Shield,
        26 => WearPos::Legs,
        29 => WearPos::Gloves,
        32 => WearPos::Boots,
        35 => WearPos::Ring,
        38 => WearPos::Ammo,
        _ => return,
    };

    let Some(obj) = player.worn().slot(slot) else { return };

    if player.inv().free_slots() == 0 {
        send_message!("Not enough inventory space.");
        return;
    }

    player.worn_mut().set(slot, None);
    player.inv_mut().add(obj.id, obj.amount).await;
    player.worn_mut().flush().await;
    player.appearance_mut().flush();
}
