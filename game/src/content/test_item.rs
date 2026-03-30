#[macros::on_item(id = 379, option = One)]
async fn eat_shrimp() {
    remove_item!(1);
    send_message!("You eat the shrimp.");
}
