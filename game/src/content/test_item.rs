#[macros::on_item_option(id = 379, option = 1)]
async fn eat_shrimp() {
    remove_item!(1);
    send_message!("You eat the shrimp.");
}
