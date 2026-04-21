#[macros::on_loc(id = 26972, op = Op1)]
async fn use_bank_booth() {
    super::open::open(&mut player).await;
}

#[macros::on_loc(id = 26972, op = Op2)]
async fn open_bank_booth() {
    super::open::open(&mut player).await;
}
