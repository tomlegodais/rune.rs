#[macros::on_interface(op = 1, interface = 750, component = 1)]
async fn run_energy_orb() {
    player.movement_mut().toggle_run().await;
}
