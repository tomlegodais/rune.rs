use std::sync::Arc;

use filesystem::{
    definition::{EquipmentFlag, EquipmentSlot, ItemDefinition},
    loader::ItemLoader,
};
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::obj::{self, ObjConfigRepository};
use shaku::HasComponent;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<ItemLoader> = OnceCell::new();

fn map_slot(slot: obj::EquipmentSlot) -> EquipmentSlot {
    match slot {
        obj::EquipmentSlot::Head => EquipmentSlot::Head,
        obj::EquipmentSlot::Cape => EquipmentSlot::Cape,
        obj::EquipmentSlot::Amulet => EquipmentSlot::Amulet,
        obj::EquipmentSlot::Weapon => EquipmentSlot::Weapon,
        obj::EquipmentSlot::Body => EquipmentSlot::Body,
        obj::EquipmentSlot::Shield => EquipmentSlot::Shield,
        obj::EquipmentSlot::Legs => EquipmentSlot::Legs,
        obj::EquipmentSlot::Gloves => EquipmentSlot::Gloves,
        obj::EquipmentSlot::Boots => EquipmentSlot::Boots,
        obj::EquipmentSlot::Ring => EquipmentSlot::Ring,
        obj::EquipmentSlot::Ammo => EquipmentSlot::Ammo,
    }
}

fn map_flag(flag: obj::EquipmentFlag) -> EquipmentFlag {
    match flag {
        obj::EquipmentFlag::TwoHanded => EquipmentFlag::TwoHanded,
        obj::EquipmentFlag::Sleeveless => EquipmentFlag::Sleeveless,
        obj::EquipmentFlag::Hair => EquipmentFlag::Hair,
        obj::EquipmentFlag::HairMid => EquipmentFlag::HairMid,
        obj::EquipmentFlag::HairLow => EquipmentFlag::HairLow,
        obj::EquipmentFlag::FullFace => EquipmentFlag::FullFace,
        obj::EquipmentFlag::Mask => EquipmentFlag::Mask,
    }
}

#[data_provider]
async fn load_item_definitions(ctx: &ProviderContext) -> anyhow::Result<()> {
    let mut loader = ItemLoader::load(&ctx.cache)?;

    let repo: Arc<dyn ObjConfigRepository> = ctx.persistence.resolve();
    let configs = repo.find_all().await?;

    for config in configs {
        if let Some(def) = loader.get_mut(config.obj_id) {
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
