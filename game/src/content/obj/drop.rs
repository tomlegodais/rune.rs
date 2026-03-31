#[macros::on_obj(op = Op7)]
async fn drop_obj() {
    let Some(obj) = slot_obj!() else {
        return;
    };

    clear_slot!();
    drop_to_ground!(obj.id, obj.amount);
}
