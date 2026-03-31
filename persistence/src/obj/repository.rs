use async_trait::async_trait;
pub use entity::{WearFlag, WearPos};
use sea_orm::*;
use shaku::{Component, Interface};

use super::entity;

pub struct ObjConfig {
    pub obj_id: u32,
    pub wearpos: Option<WearPos>,
    pub wearflag: Option<WearFlag>,
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
            })
            .collect())
    }
}
