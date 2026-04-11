#[macros::on_interface(op = 1, interface = 884)]
async fn select_combat_style() {
    let style = match component {
        11 => 0u8,
        12 => 1,
        13 => 2,
        14 => 3,
        _ => return,
    };
    player.combat_mut().set_combat_style(style);
    player.varp_mut().send_varp(43, style as i32).await;
}

#[macros::on_interface(op = 1, interface = 884, component = 15)]
async fn toggle_auto_retaliate() {
    let current = player.combat().auto_retaliate();
    player.combat_mut().set_auto_retaliate(!current);
    player.varp_mut().send_varp(172, current as i32).await;
}

#[macros::on_interface(op = 1, interface = 884, component = 4)]
async fn toggle_special_attack() {
    let current = player.combat().spec_enabled();
    if current {
        player.combat_mut().set_spec_enabled(false);
        player.varp_mut().send_varp(301, 0).await;
        return;
    }

    let cost = player
        .worn()
        .slot(filesystem::WearPos::Weapon)
        .and_then(|obj| crate::content::get_spec(obj.id))
        .map(|e| e.energy_cost);

    if cost.is_some_and(|c| player.combat().spec_energy() < c) {
        send_message!("You don't have enough special attack energy.");
        return;
    }

    player.combat_mut().set_spec_enabled(true);
    player.varp_mut().send_varp(301, 1).await;
}
