#[macros::on_item_option(option = 7)]
async fn drop_item() {
    let Some(item) = slot_item!() else {
        return;
    };

    clear_slot!();
    drop_to_ground!(item.id, item.amount);
}
