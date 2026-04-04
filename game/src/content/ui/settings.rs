#[macros::on_interface(op = 1, interface = 261)]
async fn settings_tab() {
    match component {
        3 => player.movement_mut().toggle_run().await,
        _ => tracing::debug!(
            "Unhandled Settings IfButton (op={:?}, component={}, slot1={})",
            op,
            component,
            slot1
        ),
    }
}
