const DEFAULT_RENDER_EMOTE: u16 = 1426;

#[derive(Clone)]
pub struct Appearance {
    pub male: bool,
    pub look: [u16; 7],
    pub colors: [u8; 5],
    pub render_emote: u16,
    pub display_name: String,
    pub combat_level: u8,
}

impl Appearance {
    pub fn from_data(display_name: &str, male: bool, look: [u16; 7], colors: [u8; 5]) -> Self {
        Self {
            male,
            look,
            colors,
            render_emote: DEFAULT_RENDER_EMOTE,
            display_name: display_name.to_string(),
            combat_level: 3,
        }
    }
}
