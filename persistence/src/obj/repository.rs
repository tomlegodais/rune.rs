use async_trait::async_trait;
pub use entity::{AmmoType, WeaponCategory, WearFlag, WearPos};
use sea_orm::*;
use shaku::{Component, Interface};

use super::entity::{self, ammo, ranged, stat, weapon, wear};

pub struct ObjWearConfig {
    pub obj_id: u32,
    pub wearpos: Option<WearPos>,
    pub wearflag: Option<WearFlag>,
    pub weight: i32,
}

pub struct ObjStatConfig {
    pub obj_id: u32,
    pub atk_stab: i16,
    pub atk_slash: i16,
    pub atk_crush: i16,
    pub atk_magic: i16,
    pub atk_ranged: i16,
    pub def_stab: i16,
    pub def_slash: i16,
    pub def_crush: i16,
    pub def_magic: i16,
    pub def_ranged: i16,
    pub str_bonus: i16,
    pub ranged_str: i16,
    pub magic_dmg: i16,
    pub prayer: i16,
}

pub struct ObjWeaponConfig {
    pub obj_id: u32,
    pub weapon_category: WeaponCategory,
    pub atk_speed: i16,
    pub atk_seq: Option<Vec<u16>>,
    pub block_seq: Option<u16>,
}

pub struct ObjRangedConfig {
    pub obj_id: u32,
    pub ammo_type: Option<AmmoType>,
    pub ammo_tier: Option<i16>,
    pub atk_range: Option<i16>,
    pub proj_gfx: Option<i16>,
    pub atk_spotanim: Option<i16>,
}

pub struct ObjAmmoConfig {
    pub obj_id: u32,
    pub ammo_type: AmmoType,
    pub ammo_tier: i16,
    pub proj_gfx: Option<i16>,
    pub atk_spotanim: Option<i16>,
}

#[async_trait]
pub trait ObjConfigRepository: Interface {
    async fn find_all_wear(&self) -> Result<Vec<ObjWearConfig>, DbErr>;
    async fn find_all_stats(&self) -> Result<Vec<ObjStatConfig>, DbErr>;
    async fn find_all_weapons(&self) -> Result<Vec<ObjWeaponConfig>, DbErr>;
    async fn find_all_ranged(&self) -> Result<Vec<ObjRangedConfig>, DbErr>;
    async fn find_all_ammo(&self) -> Result<Vec<ObjAmmoConfig>, DbErr>;
}

#[derive(Component)]
#[shaku(interface = ObjConfigRepository)]
pub struct PgObjConfigRepository {
    #[shaku(default)]
    db: DatabaseConnection,
}

#[async_trait]
impl ObjConfigRepository for PgObjConfigRepository {
    async fn find_all_wear(&self) -> Result<Vec<ObjWearConfig>, DbErr> {
        let models = wear::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ObjWearConfig {
                obj_id: m.obj_id as u32,
                wearpos: m.wearpos,
                wearflag: m.wearflag,
                weight: m.weight,
            })
            .collect())
    }

    async fn find_all_stats(&self) -> Result<Vec<ObjStatConfig>, DbErr> {
        let models = stat::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ObjStatConfig {
                obj_id: m.obj_id as u32,
                atk_stab: m.atk_stab,
                atk_slash: m.atk_slash,
                atk_crush: m.atk_crush,
                atk_magic: m.atk_magic,
                atk_ranged: m.atk_ranged,
                def_stab: m.def_stab,
                def_slash: m.def_slash,
                def_crush: m.def_crush,
                def_magic: m.def_magic,
                def_ranged: m.def_ranged,
                str_bonus: m.str_bonus,
                ranged_str: m.ranged_str,
                magic_dmg: m.magic_dmg,
                prayer: m.prayer,
            })
            .collect())
    }

    async fn find_all_weapons(&self) -> Result<Vec<ObjWeaponConfig>, DbErr> {
        let models = weapon::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ObjWeaponConfig {
                obj_id: m.obj_id as u32,
                weapon_category: m.weapon_category,
                atk_speed: m.atk_speed,
                atk_seq: m.atk_seq.map(|v| v.into_iter().map(|s| s as u16).collect()),
                block_seq: m.block_seq.map(|v| v as u16),
            })
            .collect())
    }

    async fn find_all_ranged(&self) -> Result<Vec<ObjRangedConfig>, DbErr> {
        let models = ranged::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ObjRangedConfig {
                obj_id: m.obj_id as u32,
                ammo_type: m.ammo_type,
                ammo_tier: m.ammo_tier,
                atk_range: m.atk_range,
                proj_gfx: m.proj_gfx,
                atk_spotanim: m.atk_spotanim,
            })
            .collect())
    }

    async fn find_all_ammo(&self) -> Result<Vec<ObjAmmoConfig>, DbErr> {
        let models = ammo::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ObjAmmoConfig {
                obj_id: m.obj_id as u32,
                ammo_type: m.ammo_type,
                ammo_tier: m.ammo_tier,
                proj_gfx: m.proj_gfx,
                atk_spotanim: m.atk_spotanim,
            })
            .collect())
    }
}
