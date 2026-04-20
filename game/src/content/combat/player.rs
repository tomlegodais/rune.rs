use filesystem::{AttackType, WeaponCategory, WeaponStance, XpType};

use super::formula::{AttackRoll, DefenceRoll, atk_bonus_for_type, def_bonus_for_type};
use crate::{
    player::{Stat, active_player},
    provider,
};

const DEFAULT_ATK_SPEED: u16 = 4;

pub struct ResolvedStyle {
    pub atk_type: AttackType,
    pub stance: WeaponStance,
    pub xp_type: XpType,
}

pub fn resolve_style(player: &crate::player::Player) -> ResolvedStyle {
    let cat = player.worn().weapon_category().unwrap_or(WeaponCategory::Unarmed);
    let styles = cat.combat_styles();
    let idx = player.combat().combat_style() as usize;
    let style = styles.get(idx).or_else(|| styles.first()).unwrap();
    ResolvedStyle {
        atk_type: style.atk_type.unwrap_or(AttackType::Crush),
        stance: style.stance.unwrap_or(WeaponStance::Accurate),
        xp_type: style.xp_type.unwrap_or(XpType::Attack),
    }
}

pub fn is_ranged_weapon(player: &crate::player::Player) -> bool {
    matches!(
        player.worn().weapon_category(),
        Some(
            WeaponCategory::Bow
                | WeaponCategory::Crossbow
                | WeaponCategory::Thrown
                | WeaponCategory::Gun
                | WeaponCategory::Chinchompa
                | WeaponCategory::Blaster
        )
    )
}

pub fn weapon_atk_speed(player: &crate::player::Player) -> u16 {
    let base = player
        .worn()
        .slot(filesystem::WearPos::Weapon)
        .and_then(|obj| provider::get_obj_type(obj.id as u32))
        .and_then(|t| t.atk_speed)
        .unwrap_or(DEFAULT_ATK_SPEED as i16) as u16;
    let style = resolve_style(player);
    if style.stance == WeaponStance::Rapid { base.saturating_sub(1) } else { base }
}

pub fn melee_atk(player: &crate::player::Player) -> (AttackRoll, ResolvedStyle) {
    let style = resolve_style(player);
    let bonuses = player.worn().bonuses();
    let atk = AttackRoll {
        atk_level: player.stat().level(Stat::Attack) as u16,
        str_level: player.stat().level(Stat::Strength) as u16,
        atk_bonus: atk_bonus_for_type(style.atk_type, bonuses.atk_stab, bonuses.atk_slash, bonuses.atk_crush),
        str_bonus: bonuses.str_bonus,
        stance: style.stance,
    };
    (atk, style)
}

pub fn ranged_atk(player: &crate::player::Player) -> (AttackRoll, ResolvedStyle) {
    let style = resolve_style(player);
    let bonuses = player.worn().bonuses();
    let ranged_level = player.stat().level(Stat::Ranged) as u16;
    let atk = AttackRoll {
        atk_level: ranged_level,
        str_level: ranged_level,
        atk_bonus: bonuses.atk_ranged,
        str_bonus: bonuses.ranged_str,
        stance: style.stance,
    };
    (atk, style)
}

pub fn player_def(player: &crate::player::Player, atk_type: AttackType) -> DefenceRoll {
    let bonuses = player.worn().bonuses();
    let style = resolve_style(player);
    DefenceRoll {
        def_level: player.stat().level(Stat::Defence) as u16,
        def_bonus: def_bonus_for_type(
            atk_type,
            bonuses.def_stab,
            bonuses.def_slash,
            bonuses.def_crush,
            bonuses.def_ranged,
            bonuses.def_magic,
        ),
        stance: style.stance,
    }
}

pub async fn award_combat_xp(xp_type: XpType, damage: u16) {
    let base = damage as f64 * 4.0;
    let player = active_player();
    match xp_type {
        XpType::Attack => player.stat_mut().add_xp(Stat::Attack, base).await,
        XpType::Strength => player.stat_mut().add_xp(Stat::Strength, base).await,
        XpType::Defence => player.stat_mut().add_xp(Stat::Defence, base).await,
        XpType::Ranged => active_player().stat_mut().add_xp(Stat::Ranged, base).await,
        XpType::RangedAndDefence => {
            let half = base / 2.0;
            active_player().stat_mut().add_xp(Stat::Ranged, half).await;
            active_player().stat_mut().add_xp(Stat::Defence, half).await;
        }
        XpType::Shared => {
            let third = base / 3.0;
            player.stat_mut().add_xp(Stat::Attack, third).await;
            player.stat_mut().add_xp(Stat::Strength, third).await;
            player.stat_mut().add_xp(Stat::Defence, third).await;
        }
        _ => {}
    }
    active_player()
        .stat_mut()
        .add_xp(Stat::Hitpoints, damage as f64 * 1.33)
        .await;
}
