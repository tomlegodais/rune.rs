use super::CommandEntry;
use crate::player::{InterfaceManager, Player, SubInterface};
use crate::send_message;
use macros::command;

#[command(name = "ifopensub")]
async fn sub(player: &mut Player, interface: u16, fixed: u16, resizable: u16) {
    {
        let sub = SubInterface::new(interface, fixed, resizable);
        let mut mgr = player.systems.guard::<InterfaceManager>();
        mgr.open_sub(&sub).await;
    }
    send_message!(
        player,
        "Opened {} at fixed={} resizable={}.",
        interface,
        fixed,
        resizable
    );
}

#[command(name = "ifclosesub")]
async fn close_sub(player: &mut Player, interface: u16, fixed: u16, resizable: u16) {
    {
        let sub = SubInterface::new(interface, fixed, resizable);
        let mut mgr = player.systems.guard::<InterfaceManager>();
        mgr.close_sub(&sub).await;
    }
    send_message!(
        player,
        "Closed {} at fixed={} resizable={}.",
        interface,
        fixed,
        resizable
    );
}
