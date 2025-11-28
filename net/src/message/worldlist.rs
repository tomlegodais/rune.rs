#[derive(Debug)]
pub struct WorldListOutbound {
    pub full_update: bool,
    pub host: String,
    pub player_count: u16,
}
