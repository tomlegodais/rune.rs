#[derive(Debug)]
pub struct Account {
    pub _id: i64,
    pub _username: String,
    pub _password_hash: String,
    pub rights: u8,
}
