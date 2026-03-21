use net::{Outbox, OutboxExt, UpdateSkill};
use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
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
    pub fn new(outbox: Outbox) -> Self {
        let mut levels = [1u8; NUM_SKILLS];
        let mut xp = [0u32; NUM_SKILLS];

        levels[Skill::Hitpoints as usize] = 10;
        xp[Skill::Hitpoints as usize] = xp_for_level(10);

        Self { outbox, levels, xp }
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
        for i in 0..NUM_SKILLS {
            self.outbox
                .write(UpdateSkill {
                    id: i as u8,
                    level: self.levels[i],
                    xp: self.xp[i],
                })
                .await;
        }
    }

    pub async fn send_skill(&mut self, skill: Skill) {
        let i = skill as usize;
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
