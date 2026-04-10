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
    player.combat_mut().set_spec_enabled(!current);
    player.varp_mut().send_varp(301, !current as i32).await;
}
