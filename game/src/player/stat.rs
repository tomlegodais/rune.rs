use std::{future::Future, pin::Pin};

use macros::player_system;
use net::{Outbox, OutboxExt, UpdateStat};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use persistence::player::PlayerData;

use crate::{
    player::{
        PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
    },
    world::World,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum Stat {
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

const NUM_STATS: usize = 24;

pub struct StatManager {
    outbox: Outbox,
    levels: [u8; NUM_STATS],
    xp: [u32; NUM_STATS],
}

impl StatManager {
    pub fn from_data(outbox: Outbox, levels: [u8; NUM_STATS], xp: [u32; NUM_STATS]) -> Self {
        Self { outbox, levels, xp }
    }

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
        self.xp[i] = self.xp[i].saturating_add(xp as u32);
        self.levels[i] = level_for_xp(self.xp[i]);
        self.send_stat(stat).await;
    }

    pub async fn flush(&mut self) {
        let stats: Vec<_> = (0..NUM_STATS).filter_map(|i| Stat::try_from(i).ok()).collect();
        for stat in stats {
            self.send_stat(stat).await;
        }
    }

    pub async fn send_stat(&mut self, stat: Stat) {
        let i: usize = stat.into();
        self.outbox
            .write(UpdateStat {
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
impl PlayerSystem for StatManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self::from_data(ctx.outbox.clone(), ctx.player_data.levels, ctx.player_data.xp)
    }

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.flush())
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.levels = self.levels();
        data.xp = self.xp_values();
    }
}
