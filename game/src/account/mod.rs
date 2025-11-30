#[derive(Debug)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub _password_hash: String,
    pub rights: u8,
}
