use async_trait::async_trait;
use sea_orm::*;
use shaku::{Component, Interface};
use spawn::FaceDirection;

use super::entity::{config, spawn};

pub struct NpcConfig {
    pub npc_id: u32,
    pub max_hp: u32,
    pub atk_level: u16,
    pub str_level: u16,
    pub def_level: u16,
    pub atk_bonus: i16,
    pub str_bonus: i16,
    pub def_stab: i16,
    pub def_slash: i16,
    pub def_crush: i16,
    pub def_magic: i16,
    pub def_ranged: i16,
    pub atk_speed: u16,
    pub atk_seq: u16,
    pub block_seq: u16,
    pub death_seq: u16,
    pub max_hit: u16,
    pub atk_range: u16,
}

pub struct NpcSpawn {
    pub npc_id: u32,
    pub x: i32,
    pub y: i32,
    pub plane: i32,
    pub wander_radius: u8,
    pub respawn_ticks: u16,
    pub face_direction: FaceDirection,
}

#[async_trait]
pub trait NpcConfigRepository: Interface {
    async fn find_all_configs(&self) -> Result<Vec<NpcConfig>, DbErr>;
    async fn find_all_spawns(&self) -> Result<Vec<NpcSpawn>, DbErr>;
}

#[derive(Component)]
#[shaku(interface = NpcConfigRepository)]
pub struct PgNpcConfigRepository {
    #[shaku(default)]
    db: DatabaseConnection,
}

#[async_trait]
impl NpcConfigRepository for PgNpcConfigRepository {
    async fn find_all_configs(&self) -> Result<Vec<NpcConfig>, DbErr> {
        let models = config::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| NpcConfig {
                npc_id: m.npc_id as u32,
                max_hp: m.max_hp as u32,
                atk_level: m.atk_level as u16,
                str_level: m.str_level as u16,
                def_level: m.def_level as u16,
                atk_bonus: m.atk_bonus,
                str_bonus: m.str_bonus,
                def_stab: m.def_stab,
                def_slash: m.def_slash,
                def_crush: m.def_crush,
                def_magic: m.def_magic,
                def_ranged: m.def_ranged,
                atk_speed: m.atk_speed as u16,
                atk_seq: m.atk_seq as u16,
                block_seq: m.block_seq as u16,
                death_seq: m.death_seq as u16,
                max_hit: m.max_hit as u16,
                atk_range: m.atk_range as u16,
            })
            .collect())
    }

    async fn find_all_spawns(&self) -> Result<Vec<NpcSpawn>, DbErr> {
        let models = spawn::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| NpcSpawn {
                npc_id: m.npc_id as u32,
                x: m.x,
                y: m.y,
                plane: m.plane,
                wander_radius: m.wander_radius as u8,
                respawn_ticks: m.respawn_ticks as u16,
                face_direction: m.face_direction,
            })
            .collect())
    }
}
