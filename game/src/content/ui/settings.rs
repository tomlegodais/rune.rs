#[macros::on_interface(op = 1, interface = 261, component = 3)]
async fn run_toggle() {
    player.movement_mut().toggle_run().await;
}
