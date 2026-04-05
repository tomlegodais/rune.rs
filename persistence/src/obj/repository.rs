use async_trait::async_trait;
pub use entity::{WearFlag, WearPos, WeaponCategory};
use sea_orm::*;
use shaku::{Component, Interface};

use super::entity;

pub struct ObjConfig {
    pub obj_id: u32,
    pub wearpos: Option<WearPos>,
    pub wearflag: Option<WearFlag>,
    pub weapon_category: Option<WeaponCategory>,
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
    pub atk_speed: Option<i16>,
    pub weight: i32,
}

#[async_trait]
pub trait ObjConfigRepository: Interface {
    async fn find_all(&self) -> Result<Vec<ObjConfig>, DbErr>;
}

#[derive(Component)]
#[shaku(interface = ObjConfigRepository)]
pub struct PgObjConfigRepository {
    #[shaku(default)]
    db: DatabaseConnection,
}

#[async_trait]
impl ObjConfigRepository for PgObjConfigRepository {
    async fn find_all(&self) -> Result<Vec<ObjConfig>, DbErr> {
        let models = entity::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ObjConfig {
                obj_id: m.obj_id as u32,
                wearpos: m.wearpos,
                wearflag: m.wearflag,
                weapon_category: m.weapon_category,
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
                atk_speed: m.atk_speed,
                weight: m.weight,
            })
            .collect())
    }
}
