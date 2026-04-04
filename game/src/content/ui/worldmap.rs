use crate::player::Clientbound;

async fn open_worldmap(player: &mut crate::player::Player) {
    let bits = player.position.to_bits();
    player.if_open_top(755, 0).await;
    player.varc_large(622, bits).await;
    player.varc_large(674, bits).await;
}

#[macros::on_interface(op = 1, interface = 548, component = 130)]
async fn worldmap_fixed() {
    open_worldmap(&mut player).await;
}

#[macros::on_interface(op = 1, interface = 746, component = 174)]
async fn worldmap_resizable() {
    open_worldmap(&mut player).await;
}

#[macros::on_interface(op = 1, interface = 755, component = 47)]
async fn close_worldmap() {
    player.interface_mut().restore_top(2).await;
}
