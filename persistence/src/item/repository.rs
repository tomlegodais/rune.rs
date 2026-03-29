use async_trait::async_trait;
pub use entity::{EquipmentFlag, EquipmentSlot};
use sea_orm::*;
use shaku::{Component, Interface};

use super::entity;

pub struct ItemConfig {
    pub item_id: u32,
    pub equipment_slot: Option<EquipmentSlot>,
    pub equipment_flag: Option<EquipmentFlag>,
}

#[async_trait]
pub trait ItemConfigRepository: Interface {
    async fn find_all(&self) -> Result<Vec<ItemConfig>, DbErr>;
}

#[derive(Component)]
#[shaku(interface = ItemConfigRepository)]
pub struct PgItemConfigRepository {
    #[shaku(default)]
    db: DatabaseConnection,
}

#[async_trait]
impl ItemConfigRepository for PgItemConfigRepository {
    async fn find_all(&self) -> Result<Vec<ItemConfig>, DbErr> {
        let models = entity::Entity::find().all(&self.db).await?;
        Ok(models
            .into_iter()
            .map(|m| ItemConfig {
                item_id: m.item_id as u32,
                equipment_slot: m.equipment_slot,
                equipment_flag: m.equipment_flag,
            })
            .collect())
    }
}
