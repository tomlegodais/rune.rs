use std::collections::HashMap;

use filesystem::{config::NpcType, loader::NpcLoader};
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::npc::{FaceDirection, NpcConfigRepository};
use shaku::HasComponent;

use crate::{npc::NpcCombat, provider::ProviderContext, world::Direction};

static INSTANCE: OnceCell<NpcLoader> = OnceCell::new();
static CONFIGS: OnceCell<HashMap<u32, NpcCombat>> = OnceCell::new();
static SPAWNS: OnceCell<Vec<NpcSpawnDef>> = OnceCell::new();

pub struct NpcSpawnDef {
    pub npc_id: u16,
    pub x: i32,
    pub y: i32,
    pub plane: i32,
    pub wander_radius: u8,
    pub respawn_ticks: u16,
    pub face_direction: Direction,
}

#[data_provider]
async fn load_npc_types(ctx: &ProviderContext) -> anyhow::Result<()> {
    Ok(INSTANCE.get_or_try_init(|| NpcLoader::load(&ctx.cache)).map(drop)?)
}

#[data_provider]
async fn load_npc_configs(ctx: &ProviderContext) -> anyhow::Result<()> {
    let repo: &dyn NpcConfigRepository = ctx.persistence.resolve_ref();
    let configs = repo.find_all_configs().await?;
    let spawns = repo.find_all_spawns().await?;

    let combat_map: HashMap<u32, NpcCombat> = configs
        .into_iter()
        .map(|c| {
            (
                c.npc_id,
                NpcCombat {
                    max_hp: c.max_hp,
                    atk_level: c.atk_level,
                    str_level: c.str_level,
                    def_level: c.def_level,
                    atk_bonus: c.atk_bonus,
                    str_bonus: c.str_bonus,
                    def_stab: c.def_stab,
                    def_slash: c.def_slash,
                    def_crush: c.def_crush,
                    atk_speed: c.atk_speed,
                    atk_seq: c.atk_seq,
                    block_seq: c.block_seq,
                    death_seq: c.death_seq,
                    max_hit: c.max_hit,
                },
            )
        })
        .collect();

    let spawn_defs: Vec<NpcSpawnDef> = spawns
        .into_iter()
        .map(|s| NpcSpawnDef {
            npc_id: s.npc_id as u16,
            x: s.x,
            y: s.y,
            plane: s.plane,
            wander_radius: s.wander_radius,
            respawn_ticks: s.respawn_ticks,
            face_direction: s.face_direction.into(),
        })
        .collect();

    let _ = CONFIGS.set(combat_map);
    let _ = SPAWNS.set(spawn_defs);
    Ok(())
}

impl From<FaceDirection> for Direction {
    fn from(fd: FaceDirection) -> Self {
        match fd {
            FaceDirection::North => Self::North,
            FaceDirection::NorthEast => Self::NorthEast,
            FaceDirection::East => Self::East,
            FaceDirection::SouthEast => Self::SouthEast,
            FaceDirection::South => Self::South,
            FaceDirection::SouthWest => Self::SouthWest,
            FaceDirection::West => Self::West,
            FaceDirection::NorthWest => Self::NorthWest,
        }
    }
}

pub fn get_npc_type(id: u32) -> Option<&'static NpcType> {
    INSTANCE.get().and_then(|l| l.get(id))
}

pub fn get_npc_combat(id: u32) -> Option<&'static NpcCombat> {
    CONFIGS.get().and_then(|m| m.get(&id))
}

pub fn get_npc_spawns() -> &'static [NpcSpawnDef] {
    SPAWNS.get().map(|v| v.as_slice()).unwrap_or(&[])
}
