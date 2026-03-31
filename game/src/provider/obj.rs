use std::sync::Arc;

use filesystem::{
    definition::{ObjType, WearFlag, WearPos},
    loader::ObjLoader,
};
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::obj::{self, ObjConfigRepository};
use shaku::HasComponent;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<ObjLoader> = OnceCell::new();

fn map_wearpos(slot: obj::WearPos) -> WearPos {
    match slot {
        obj::WearPos::Head => WearPos::Head,
        obj::WearPos::Cape => WearPos::Cape,
        obj::WearPos::Amulet => WearPos::Amulet,
        obj::WearPos::Weapon => WearPos::Weapon,
        obj::WearPos::Body => WearPos::Body,
        obj::WearPos::Shield => WearPos::Shield,
        obj::WearPos::Legs => WearPos::Legs,
        obj::WearPos::Gloves => WearPos::Gloves,
        obj::WearPos::Boots => WearPos::Boots,
        obj::WearPos::Ring => WearPos::Ring,
        obj::WearPos::Ammo => WearPos::Ammo,
    }
}

fn map_wearflag(flag: obj::WearFlag) -> WearFlag {
    match flag {
        obj::WearFlag::TwoHanded => WearFlag::TwoHanded,
        obj::WearFlag::Sleeveless => WearFlag::Sleeveless,
        obj::WearFlag::Hair => WearFlag::Hair,
        obj::WearFlag::HairMid => WearFlag::HairMid,
        obj::WearFlag::HairLow => WearFlag::HairLow,
        obj::WearFlag::FullFace => WearFlag::FullFace,
        obj::WearFlag::Mask => WearFlag::Mask,
    }
}

#[data_provider]
async fn load_obj_types(ctx: &ProviderContext) -> anyhow::Result<()> {
    let mut loader = ObjLoader::load(&ctx.cache)?;

    let repo: Arc<dyn ObjConfigRepository> = ctx.persistence.resolve();
    let configs = repo.find_all().await?;

    for config in configs {
        if let Some(t) = loader.get_mut(config.obj_id) {
            t.wearpos = config.wearpos.map(map_wearpos);
            t.wearflag = config.wearflag.map(map_wearflag).unwrap_or_default();
        }
    }

    INSTANCE
        .set(loader)
        .map_err(|_| anyhow::anyhow!("obj types already loaded"))
}

pub fn get_obj_type(id: u32) -> Option<&'static ObjType> {
    INSTANCE.get().and_then(|l| l.get(id))
}
