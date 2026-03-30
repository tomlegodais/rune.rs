#[macros::on_item(option = Seven)]
async fn drop_item() {
    let Some(item) = slot_item!() else {
        return;
    };

    clear_slot!();
    drop_to_ground!(item.id, item.amount);
}
