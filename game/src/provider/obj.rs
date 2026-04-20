use std::sync::Arc;

use filesystem::{AmmoType, EquipBonuses, ObjLoader, ObjType, WeaponCategory, WearFlag, WearPos};
use macros::data_provider;
use once_cell::sync::OnceCell;
use persistence::{DbAmmoType, DbWeaponCategory, DbWearFlag, DbWearPos, ObjConfigRepository};
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

fn map_ammo_type(t: DbAmmoType) -> AmmoType {
    match t {
        DbAmmoType::Arrow => AmmoType::Arrow,
        DbAmmoType::Bolt => AmmoType::Bolt,
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
    let wear_configs = repo.find_all_wear().await?;
    let stat_configs = repo.find_all_stats().await?;
    let weapon_configs = repo.find_all_weapons().await?;
    let ranged_configs = repo.find_all_ranged().await?;
    let ammo_configs = repo.find_all_ammo().await?;

    for cfg in wear_configs {
        if let Some(t) = loader.get_mut(cfg.obj_id) {
            t.wearpos = cfg.wearpos.map(map_wearpos);
            t.wearflag = cfg.wearflag.map(map_wearflag).unwrap_or_default();
            t.weight = cfg.weight;
        }
    }

    for cfg in stat_configs {
        if let Some(t) = loader.get_mut(cfg.obj_id) {
            t.equip = EquipBonuses {
                atk_stab: cfg.atk_stab,
                atk_slash: cfg.atk_slash,
                atk_crush: cfg.atk_crush,
                atk_magic: cfg.atk_magic,
                atk_ranged: cfg.atk_ranged,
                def_stab: cfg.def_stab,
                def_slash: cfg.def_slash,
                def_crush: cfg.def_crush,
                def_magic: cfg.def_magic,
                def_ranged: cfg.def_ranged,
                str_bonus: cfg.str_bonus,
                ranged_str: cfg.ranged_str,
                magic_dmg: cfg.magic_dmg,
                prayer: cfg.prayer,
            };
        }
    }

    for cfg in weapon_configs {
        if let Some(t) = loader.get_mut(cfg.obj_id) {
            t.weapon_category = Some(map_weapon_category(cfg.weapon_category));
            t.atk_speed = Some(cfg.atk_speed);
            if let Some(seq) = cfg.atk_seq {
                t.atk_seq = seq;
            }
            t.block_seq = cfg.block_seq;
        }
    }

    for cfg in ranged_configs {
        if let Some(t) = loader.get_mut(cfg.obj_id) {
            t.ammo_type = cfg.ammo_type.map(map_ammo_type);
            t.ammo_tier = cfg.ammo_tier;
            t.atk_range = cfg.atk_range;
            t.proj_gfx = cfg.proj_gfx.map(|v| v as u16);
            t.atk_spotanim = cfg.atk_spotanim.map(|v| v as u16);
        }
    }

    for cfg in ammo_configs {
        if let Some(t) = loader.get_mut(cfg.obj_id) {
            t.ammo_type = Some(map_ammo_type(cfg.ammo_type));
            t.ammo_tier = Some(cfg.ammo_tier);
            t.proj_gfx = cfg.proj_gfx.map(|v| v as u16);
            t.atk_spotanim = cfg.atk_spotanim.map(|v| v as u16);
        }
    }

    INSTANCE
        .set(loader)
        .map_err(|_| anyhow::anyhow!("obj types already loaded"))
}

pub fn get_obj_type(id: u32) -> Option<&'static ObjType> {
    INSTANCE.get().and_then(|l| l.get(id))
}
