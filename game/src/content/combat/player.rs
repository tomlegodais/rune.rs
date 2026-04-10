use filesystem::{AttackType, WeaponCategory, WeaponStance, XpType};

use super::formula::{MeleeAttack, MeleeDefence, atk_bonus_for_type, def_bonus_for_type};
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

pub fn weapon_atk_speed(player: &crate::player::Player) -> u16 {
    player
        .worn()
        .slot(filesystem::WearPos::Weapon)
        .and_then(|obj| provider::get_obj_type(obj.id as u32))
        .and_then(|t| t.atk_speed)
        .unwrap_or(DEFAULT_ATK_SPEED as i16) as u16
}

pub fn melee_atk(player: &crate::player::Player) -> (MeleeAttack, ResolvedStyle) {
    let style = resolve_style(player);
    let bonuses = player.worn().bonuses();
    let atk = MeleeAttack {
        atk_level: player.stat().level(Stat::Attack) as u16,
        str_level: player.stat().level(Stat::Strength) as u16,
        atk_bonus: atk_bonus_for_type(style.atk_type, bonuses.atk_stab, bonuses.atk_slash, bonuses.atk_crush),
        str_bonus: bonuses.str_bonus,
        stance: style.stance,
    };
    (atk, style)
}

pub fn melee_def(player: &crate::player::Player, atk_type: AttackType) -> MeleeDefence {
    let bonuses = player.worn().bonuses();
    let style = resolve_style(player);
    MeleeDefence {
        def_level: player.stat().level(Stat::Defence) as u16,
        def_bonus: def_bonus_for_type(atk_type, bonuses.def_stab, bonuses.def_slash, bonuses.def_crush),
        stance: style.stance,
    }
}

pub async fn award_melee_xp(xp_type: XpType, damage: u16) {
    let base = damage as f64 * 4.0;
    let player = active_player();
    match xp_type {
        XpType::Attack => player.stat_mut().add_xp(Stat::Attack, base).await,
        XpType::Strength => player.stat_mut().add_xp(Stat::Strength, base).await,
        XpType::Defence => player.stat_mut().add_xp(Stat::Defence, base).await,
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
