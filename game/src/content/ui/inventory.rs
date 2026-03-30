use crate::handler::try_dispatch_item;

#[macros::on_interface(interface = 149, component = 0)]
async fn inventory_item() {
    if !try_dispatch_item(&mut player, option, slot1) {
        send_message!("Nothing interesting happens.");
    }
}
