#[macros::on_interface(op = 1, interface = 182, component = 6)]
async fn logout_tab() {
    player.logout().await;
}
