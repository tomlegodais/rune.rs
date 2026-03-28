use crate::player::system::{PlayerInitContext, PlayerSystem, SystemContext};
use macros::player_system;
use net::{Outbox, OutboxExt, UpdateSkill};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use persistence::player::PlayerData;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum Skill {
    Attack = 0,
    Defence = 1,
    Strength = 2,
    Hitpoints = 3,
    Ranged = 4,
    Prayer = 5,
    Magic = 6,
    Cooking = 7,
    Woodcutting = 8,
    Fletching = 9,
    Fishing = 10,
    Firemaking = 11,
    Crafting = 12,
    Smithing = 13,
    Mining = 14,
    Herblore = 15,
    Agility = 16,
    Thieving = 17,
    Slayer = 18,
    Farming = 19,
    Runecraft = 20,
    Hunter = 21,
    Construction = 22,
    Summoning = 23,
}

const NUM_SKILLS: usize = 24;

pub struct SkillManager {
    outbox: Outbox,
    levels: [u8; NUM_SKILLS],
    xp: [u32; NUM_SKILLS],
}

impl SkillManager {
    pub fn from_data(outbox: Outbox, levels: [u8; NUM_SKILLS], xp: [u32; NUM_SKILLS]) -> Self {
        Self { outbox, levels, xp }
    }

    pub fn levels(&self) -> [u8; NUM_SKILLS] {
        self.levels
    }

    pub fn xp_values(&self) -> [u32; NUM_SKILLS] {
        self.xp
    }

    pub fn level(&self, skill: Skill) -> u8 {
        self.levels[skill as usize]
    }

    pub fn xp(&self, skill: Skill) -> u32 {
        self.xp[skill as usize]
    }

    pub fn set_level(&mut self, skill: Skill, level: u8) {
        let i = skill as usize;
        self.levels[i] = level;
        self.xp[i] = xp_for_level(level);
    }

    pub fn set_xp(&mut self, skill: Skill, xp: u32) {
        let i = skill as usize;
        self.xp[i] = xp;
        self.levels[i] = level_for_xp(xp);
    }

    pub async fn flush(&mut self) {
        let skills: Vec<_> = (0..NUM_SKILLS)
            .filter_map(|i| Skill::try_from(i).ok())
            .collect();
        for skill in skills {
            self.send_skill(skill).await;
        }
    }

    pub async fn send_skill(&mut self, skill: Skill) {
        let i: usize = skill.into();
        self.outbox
            .write(UpdateSkill {
                id: i as u8,
                level: self.levels[i],
                xp: self.xp[i],
            })
            .await;
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
impl PlayerSystem for SkillManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self::from_data(ctx.outbox.clone(), ctx.data.levels, ctx.data.xp)
    }

    fn on_login<'a>(
        &'a mut self,
        _ctx: &'a mut SystemContext<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.flush())
    }

    fn tick_context(_: &std::sync::Arc<crate::world::World>, _: &crate::player::PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.levels = self.levels();
        data.xp = self.xp_values();
    }
}
