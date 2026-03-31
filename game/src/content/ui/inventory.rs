use crate::handler::try_dispatch_obj;

#[macros::on_interface(interface = 149, component = 0)]
async fn inventory_item() {
    if !try_dispatch_obj(&mut player, op, slot1) {
        send_message!("Nothing interesting happens.");
    }
}
