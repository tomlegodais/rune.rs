#[macros::on_item_option(option = 7)]
async fn drop_item() {
    let Some((item_id, amount)) = slot_item!() else {
        return;
    };

    clear_slot!();
    drop_to_ground!(item_id, amount);
}
