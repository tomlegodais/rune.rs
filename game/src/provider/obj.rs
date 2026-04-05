use std::sync::Arc;

use filesystem::config::{
    EquipBonuses, ObjType, WearFlag, WearPos, WeaponCategory,
};
use filesystem::loader::ObjLoader;
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

fn map_weapon_category(cat: obj::WeaponCategory) -> WeaponCategory {
    match cat {
        obj::WeaponCategory::TwoHandedSword => WeaponCategory::TwoHandedSword,
        obj::WeaponCategory::Axe => WeaponCategory::Axe,
        obj::WeaponCategory::Banner => WeaponCategory::Banner,
        obj::WeaponCategory::Blunt => WeaponCategory::Blunt,
        obj::WeaponCategory::Bludgeon => WeaponCategory::Bludgeon,
        obj::WeaponCategory::Bulwark => WeaponCategory::Bulwark,
        obj::WeaponCategory::Claw => WeaponCategory::Claw,
        obj::WeaponCategory::Egg => WeaponCategory::Egg,
        obj::WeaponCategory::Partisan => WeaponCategory::Partisan,
        obj::WeaponCategory::Pickaxe => WeaponCategory::Pickaxe,
        obj::WeaponCategory::Polearm => WeaponCategory::Polearm,
        obj::WeaponCategory::Polestaff => WeaponCategory::Polestaff,
        obj::WeaponCategory::Scythe => WeaponCategory::Scythe,
        obj::WeaponCategory::SlashSword => WeaponCategory::SlashSword,
        obj::WeaponCategory::Spear => WeaponCategory::Spear,
        obj::WeaponCategory::Spiked => WeaponCategory::Spiked,
        obj::WeaponCategory::StabSword => WeaponCategory::StabSword,
        obj::WeaponCategory::Unarmed => WeaponCategory::Unarmed,
        obj::WeaponCategory::Whip => WeaponCategory::Whip,
        obj::WeaponCategory::Bow => WeaponCategory::Bow,
        obj::WeaponCategory::Blaster => WeaponCategory::Blaster,
        obj::WeaponCategory::Chinchompa => WeaponCategory::Chinchompa,
        obj::WeaponCategory::Crossbow => WeaponCategory::Crossbow,
        obj::WeaponCategory::Gun => WeaponCategory::Gun,
        obj::WeaponCategory::Thrown => WeaponCategory::Thrown,
        obj::WeaponCategory::BladedStaff => WeaponCategory::BladedStaff,
        obj::WeaponCategory::PoweredStaff => WeaponCategory::PoweredStaff,
        obj::WeaponCategory::Staff => WeaponCategory::Staff,
        obj::WeaponCategory::Salamander => WeaponCategory::Salamander,
        obj::WeaponCategory::MultiStyle => WeaponCategory::MultiStyle,
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
            t.weapon_category = config.weapon_category.map(map_weapon_category);
            t.atk_speed = config.atk_speed;
            t.weight = config.weight;
            t.equip = EquipBonuses {
                atk_stab: config.atk_stab,
                atk_slash: config.atk_slash,
                atk_crush: config.atk_crush,
                atk_magic: config.atk_magic,
                atk_ranged: config.atk_ranged,
                def_stab: config.def_stab,
                def_slash: config.def_slash,
                def_crush: config.def_crush,
                def_magic: config.def_magic,
                def_ranged: config.def_ranged,
                str_bonus: config.str_bonus,
                ranged_str: config.ranged_str,
                magic_dmg: config.magic_dmg,
                prayer: config.prayer,
            };
        }
    }

    INSTANCE
        .set(loader)
        .map_err(|_| anyhow::anyhow!("obj types already loaded"))
}

pub fn get_obj_type(id: u32) -> Option<&'static ObjType> {
    INSTANCE.get().and_then(|l| l.get(id))
}
