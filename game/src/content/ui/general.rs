#[macros::on_interface(op = 1, interface = 182, component = 6)]
async fn logout_tab() {
    player.logout().await;
}

#[macros::on_interface(op = 1, interface = 750, component = 1)]
async fn run_energy_orb() {
    player.toggle_run().await;
}

#[macros::on_interface(op = 1, interface = 261)]
async fn settings_tab() {
    match component {
        3 => player.toggle_run().await,
        _ => tracing::info!(
            "Unhandled Settings Tab Button (op={:?}, component={}, slot1={})",
            op,
            component,
            slot1
        ),
    }
}
