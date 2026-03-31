#[macros::on_obj(option = Seven)]
async fn drop_item() {
    let Some(obj) = slot_obj!() else {
        return;
    };

    clear_slot!();
    drop_to_ground!(obj.id, obj.amount);
}
