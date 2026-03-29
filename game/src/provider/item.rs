use std::sync::Arc;

use filesystem::{
    definition::{EquipmentFlag, EquipmentSlot, ItemDefinition},
    loader::ItemLoader,
};
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::item::{self, ItemConfigRepository};
use shaku::HasComponent;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<ItemLoader> = OnceCell::new();

fn map_slot(slot: item::EquipmentSlot) -> EquipmentSlot {
    match slot {
        item::EquipmentSlot::Head => EquipmentSlot::Head,
        item::EquipmentSlot::Cape => EquipmentSlot::Cape,
        item::EquipmentSlot::Amulet => EquipmentSlot::Amulet,
        item::EquipmentSlot::Weapon => EquipmentSlot::Weapon,
        item::EquipmentSlot::Body => EquipmentSlot::Body,
        item::EquipmentSlot::Shield => EquipmentSlot::Shield,
        item::EquipmentSlot::Legs => EquipmentSlot::Legs,
        item::EquipmentSlot::Gloves => EquipmentSlot::Gloves,
        item::EquipmentSlot::Boots => EquipmentSlot::Boots,
        item::EquipmentSlot::Ring => EquipmentSlot::Ring,
        item::EquipmentSlot::Ammo => EquipmentSlot::Ammo,
    }
}

fn map_flag(flag: item::EquipmentFlag) -> EquipmentFlag {
    match flag {
        item::EquipmentFlag::TwoHanded => EquipmentFlag::TwoHanded,
        item::EquipmentFlag::Sleeveless => EquipmentFlag::Sleeveless,
        item::EquipmentFlag::Hair => EquipmentFlag::Hair,
        item::EquipmentFlag::HairMid => EquipmentFlag::HairMid,
        item::EquipmentFlag::HairLow => EquipmentFlag::HairLow,
        item::EquipmentFlag::FullFace => EquipmentFlag::FullFace,
        item::EquipmentFlag::Mask => EquipmentFlag::Mask,
    }
}

#[data_provider]
async fn load_item_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    let mut loader = ItemLoader::load(&ctx.cache)?;

    let repo: Arc<dyn ItemConfigRepository> = ctx.persistence.resolve();
    let configs = repo.find_all().await?;

    for config in configs {
        if let Some(def) = loader.get_mut(config.item_id) {
            def.equipment_slot = config.equipment_slot.map(map_slot);
            def.equipment_flag = config.equipment_flag.map(map_flag).unwrap_or_default();
        }
    }

    INSTANCE
        .set(loader)
        .map_err(|_| anyhow::anyhow!("item definitions already loaded"))
}

pub fn get_item_definition(id: u32) -> Option<&'static ItemDefinition> {
    INSTANCE.get().and_then(|l| l.get(id))
}
