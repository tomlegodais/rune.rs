const DEFAULT_LOOK: [u16; 7] = [8, 14, 18, 26, 34, 38, 42];
const DEFAULT_COLORS: [u8; 5] = [5, 15, 15, 0, 0];
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
    pub fn new(display_name: &str, combat_level: u8) -> Self {
        Self {
            male: true,
            look: DEFAULT_LOOK,
            colors: DEFAULT_COLORS,
            render_emote: DEFAULT_RENDER_EMOTE,
            display_name: display_name.to_string(),
            combat_level,
        }
    }
}
