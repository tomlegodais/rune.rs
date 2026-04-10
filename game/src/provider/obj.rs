use std::sync::Arc;

use filesystem::{EquipBonuses, ObjLoader, ObjType, WeaponCategory, WearFlag, WearPos};
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::{DbWeaponCategory, DbWearFlag, DbWearPos, ObjConfigRepository};
use shaku::HasComponent;

use crate::provider::ProviderContext;

static INSTANCE: OnceCell<ObjLoader> = OnceCell::new();

fn map_wearpos(slot: DbWearPos) -> WearPos {
    match slot {
        DbWearPos::Head => WearPos::Head,
        DbWearPos::Cape => WearPos::Cape,
        DbWearPos::Amulet => WearPos::Amulet,
        DbWearPos::Weapon => WearPos::Weapon,
        DbWearPos::Body => WearPos::Body,
        DbWearPos::Shield => WearPos::Shield,
        DbWearPos::Legs => WearPos::Legs,
        DbWearPos::Gloves => WearPos::Gloves,
        DbWearPos::Boots => WearPos::Boots,
        DbWearPos::Ring => WearPos::Ring,
        DbWearPos::Ammo => WearPos::Ammo,
    }
}

fn map_wearflag(flag: DbWearFlag) -> WearFlag {
    match flag {
        DbWearFlag::TwoHanded => WearFlag::TwoHanded,
        DbWearFlag::Sleeveless => WearFlag::Sleeveless,
        DbWearFlag::Hair => WearFlag::Hair,
        DbWearFlag::HairMid => WearFlag::HairMid,
        DbWearFlag::HairLow => WearFlag::HairLow,
        DbWearFlag::FullFace => WearFlag::FullFace,
        DbWearFlag::Mask => WearFlag::Mask,
    }
}

fn map_weapon_category(cat: DbWeaponCategory) -> WeaponCategory {
    match cat {
        DbWeaponCategory::TwoHandedSword => WeaponCategory::TwoHandedSword,
        DbWeaponCategory::Axe => WeaponCategory::Axe,
        DbWeaponCategory::Banner => WeaponCategory::Banner,
        DbWeaponCategory::Blunt => WeaponCategory::Blunt,
        DbWeaponCategory::Bludgeon => WeaponCategory::Bludgeon,
        DbWeaponCategory::Bulwark => WeaponCategory::Bulwark,
        DbWeaponCategory::Claw => WeaponCategory::Claw,
        DbWeaponCategory::Egg => WeaponCategory::Egg,
        DbWeaponCategory::Partisan => WeaponCategory::Partisan,
        DbWeaponCategory::Pickaxe => WeaponCategory::Pickaxe,
        DbWeaponCategory::Polearm => WeaponCategory::Polearm,
        DbWeaponCategory::Polestaff => WeaponCategory::Polestaff,
        DbWeaponCategory::Scythe => WeaponCategory::Scythe,
        DbWeaponCategory::SlashSword => WeaponCategory::SlashSword,
        DbWeaponCategory::Spear => WeaponCategory::Spear,
        DbWeaponCategory::Spiked => WeaponCategory::Spiked,
        DbWeaponCategory::StabSword => WeaponCategory::StabSword,
        DbWeaponCategory::Unarmed => WeaponCategory::Unarmed,
        DbWeaponCategory::Whip => WeaponCategory::Whip,
        DbWeaponCategory::Bow => WeaponCategory::Bow,
        DbWeaponCategory::Blaster => WeaponCategory::Blaster,
        DbWeaponCategory::Chinchompa => WeaponCategory::Chinchompa,
        DbWeaponCategory::Crossbow => WeaponCategory::Crossbow,
        DbWeaponCategory::Gun => WeaponCategory::Gun,
        DbWeaponCategory::Thrown => WeaponCategory::Thrown,
        DbWeaponCategory::BladedStaff => WeaponCategory::BladedStaff,
        DbWeaponCategory::PoweredStaff => WeaponCategory::PoweredStaff,
        DbWeaponCategory::Staff => WeaponCategory::Staff,
        DbWeaponCategory::Salamander => WeaponCategory::Salamander,
        DbWeaponCategory::MultiStyle => WeaponCategory::MultiStyle,
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
            if let Some(seq) = config.atk_seq {
                t.atk_seq = seq;
            }
            t.block_seq = config.block_seq;
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
