use crate::player::{InterfaceSlot::Modal, Stat};

#[macros::on_interface(op = 1, interface = 320)]
async fn skills_tab() {
    let Some(stat) = Stat::from_skill_component(component) else { return };

    let (varp, value, interface) = if player.varp().get_varbit(stat.flash_varbit()) != 0 {
        player.varp_mut().send_varbit(stat.flash_varbit(), 0).await;
        (1230, stat.lvlup_varbit(), 741)
    } else {
        (965, stat.skill_menu(), 499)
    };

    player.varp_mut().send_varp(varp, value).await;
    player.interface_mut().open_slot(Modal, interface).await;
}

#[macros::on_interface(op = 1, interface = 499)]
async fn skill_menu() {
    if (10..=25).contains(&component) {
        let base = player.varp().get(965) & 0x3FF;
        player
            .varp_mut()
            .send_varp(965, ((component as i32 - 10) * 1024) + base)
            .await;
    }
}
