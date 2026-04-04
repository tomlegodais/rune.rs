use std::{future::Future, pin::Pin};

use macros::{enum_data, player_system};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use persistence::player::PlayerData;

use crate::{
    player::{
        Clientbound, PlayerSnapshot, chatbox,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::World,
};

#[enum_data(discriminant, u32, i32, u16, i32, i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, strum::IntoStaticStr)]
#[repr(usize)]
pub enum Stat {
    Attack = (0, 4732, 1, 123, 1, 10),
    Defence = (1, 4734, 5, 125, 5, 40),
    Strength = (2, 4733, 2, 124, 2, 20),
    Hitpoints = (3, 4738, 6, 131, 6, 50),
    Ranged = (4, 4735, 3, 126, 3, 30),
    Prayer = (5, 4736, 7, 127, 7, 60),
    Magic = (6, 4737, 4, 128, 4, 33),
    Cooking = (7, 4747, 16, 142, 16, 641),
    Woodcutting = (8, 4749, 18, 144, 18, 660),
    Fletching = (9, 4743, 19, 136, 19, 665),
    Fishing = (10, 4746, 15, 141, 15, 120),
    Firemaking = (11, 4748, 17, 143, 17, 649),
    Crafting = (12, 4742, 11, 135, 11, 90),
    Smithing = (13, 4745, 14, 140, 14, 115),
    Mining = (14, 4744, 13, 139, 13, 110),
    Herblore = (15, 4740, 9, 133, 9, 75),
    Agility = (16, 4739, 8, 132, 8, 65),
    Thieving = (17, 4741, 10, 134, 10, 80),
    Slayer = (18, 4751, 20, 137, 20, 673),
    Farming = (19, 4752, 21, 145, 21, 681),
    Runecraft = (20, 4750, 12, 129, 12, 100),
    Hunter = (21, 4754, 23, 138, 23, 689),
    Construction = (22, 4753, 22, 130, 22, 698),
    Summoning = (23, 4755, 24, 146, 24, 705),
}

impl Stat {
    pub fn article(self) -> &'static str {
        let name: &str = self.into();
        if name.starts_with(['A', 'E', 'I', 'O', 'U']) { "an" } else { "a" }
    }

    pub fn flash_varbit(self) -> u32 {
        self.data().0
    }
    pub fn level_up_icon(self) -> i32 {
        self.data().1
    }
    pub fn skill_component(self) -> u16 {
        self.data().2
    }
    pub fn skill_menu(self) -> i32 {
        self.data().3
    }
    pub fn lvlup_varbit(self) -> i32 {
        self.data().4
    }
    pub fn from_skill_component(component: u16) -> Option<Self> {
        (0..NUM_STATS)
            .filter_map(|i| Self::try_from(i).ok())
            .find(|s| s.skill_component() == component)
    }
}

pub const NUM_STATS: usize = 24;

pub struct StatManager {
    player: PlayerHandle,
    levels: [u8; NUM_STATS],
    xp: [u32; NUM_STATS],
}

impl StatManager {
    pub fn levels(&self) -> [u8; NUM_STATS] {
        self.levels
    }

    pub fn xp_values(&self) -> [u32; NUM_STATS] {
        self.xp
    }

    pub fn level(&self, stat: Stat) -> u8 {
        self.levels[stat as usize]
    }

    pub fn xp(&self, stat: Stat) -> u32 {
        self.xp[stat as usize]
    }

    pub fn set_level(&mut self, stat: Stat, level: u8) {
        let i = stat as usize;
        self.levels[i] = level;
        self.xp[i] = xp_for_level(level);
    }

    pub fn set_xp(&mut self, stat: Stat, xp: u32) {
        let i = stat as usize;
        self.xp[i] = xp;
        self.levels[i] = level_for_xp(xp);
    }

    pub async fn add_xp(&mut self, stat: Stat, xp: f64) {
        let i = stat as usize;
        let old_level = self.levels[i];
        self.xp[i] = self.xp[i].saturating_add(xp as u32);
        self.levels[i] = level_for_xp(self.xp[i]);
        self.send_stat(stat).await;

        if self.levels[i] > old_level {
            self.on_level_up(stat, self.levels[i]).await;
        }
    }

    async fn on_level_up(&mut self, stat: Stat, new_level: u8) {
        let name: &str = stat.into();
        let article = stat.article();
        let line1 = format!("Congratulations, you have just advanced {} {} level!", article, name);
        let line2 = format!("You have now reached level {}.", new_level);

        self.player.spot_anim(199);
        self.player
            .dialogue_mut()
            .chatbox(chatbox::LEVEL_UP, &[&line1, &line2])
            .await;

        self.player.varp_mut().send_varbit(4757, stat.level_up_icon()).await;
        self.player.varp_mut().send_varbit(stat.flash_varbit(), 1).await;
        self.player.play_jingle(39).await;
        self.player
            .send_message(format!(
                "You've just advanced {} {} level! You have reached level {}.",
                article, name, new_level
            ))
            .await;

        self.player.appearance_mut().flush();
    }

    pub fn combat_level(&self) -> u8 {
        let l = |stat: Stat| self.levels[stat as usize] as u32;
        let base = (l(Stat::Defence) + l(Stat::Hitpoints) + l(Stat::Prayer) / 2 + l(Stat::Summoning) / 2) * 10;
        let melee = (l(Stat::Attack) + l(Stat::Strength)) * 13;
        let ranged = l(Stat::Ranged) * 3 / 2 * 13;
        let magic = l(Stat::Magic) * 3 / 2 * 13;
        ((base + melee.max(ranged).max(magic)) / 40) as u8
    }

    pub async fn flush(&mut self) {
        let stats: Vec<_> = (0..NUM_STATS).filter_map(|i| Stat::try_from(i).ok()).collect();
        for stat in stats {
            self.send_stat(stat).await;
        }
    }

    pub async fn send_stat(&mut self, stat: Stat) {
        let i: usize = stat.into();
        self.player.update_stat(i as u8, self.levels[i], self.xp[i]).await;
    }
}

fn xp_for_level(level: u8) -> u32 {
    let mut total = 0u32;
    for lvl in 1..level as u32 {
        total += (lvl as f64 + 300.0 * 2.0_f64.powf(lvl as f64 / 7.0)) as u32;
    }
    total / 4
}

fn level_for_xp(xp: u32) -> u8 {
    for level in (1..=99u8).rev() {
        if xp >= xp_for_level(level) {
            return level;
        }
    }
    1
}

#[player_system]
impl PlayerSystem for StatManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            levels: ctx.player_data.levels,
            xp: ctx.player_data.xp,
        }
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.flush())
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.levels = self.levels();
        data.xp = self.xp_values();
    }
}
