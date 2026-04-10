use filesystem::{AttackType, WeaponStance};
use rand::Rng;

pub struct MeleeAttack {
    pub atk_level: u16,
    pub str_level: u16,
    pub atk_bonus: i16,
    pub str_bonus: i16,
    pub stance: WeaponStance,
}

pub struct MeleeDefence {
    pub def_level: u16,
    pub def_bonus: i16,
    pub stance: WeaponStance,
}

fn stance_atk_bonus(stance: WeaponStance) -> u32 {
    match stance {
        WeaponStance::Accurate => 3,
        WeaponStance::Controlled => 1,
        _ => 0,
    }
}

fn stance_str_bonus(stance: WeaponStance) -> u32 {
    match stance {
        WeaponStance::Aggressive => 3,
        WeaponStance::Controlled => 1,
        _ => 0,
    }
}

fn stance_def_bonus(stance: WeaponStance) -> u32 {
    match stance {
        WeaponStance::Defensive | WeaponStance::Longrange => 3,
        WeaponStance::Controlled => 1,
        _ => 0,
    }
}

fn effective_level(base: u16, stance_bonus: u32) -> u32 {
    base as u32 + stance_bonus + 8
}

pub fn max_hit(attacker: &MeleeAttack) -> u16 {
    let eff_str = effective_level(attacker.str_level, stance_str_bonus(attacker.stance));
    let bonus = attacker.str_bonus.max(0) as u32 + 64;
    ((eff_str * bonus + 320) / 640) as u16
}

pub fn accuracy(attacker: &MeleeAttack, defender: &MeleeDefence, atk_type: AttackType) -> bool {
    let _ = atk_type; // atk_type already factored into the bonus values passed in
    let eff_atk = effective_level(attacker.atk_level, stance_atk_bonus(attacker.stance));
    let atk_roll = eff_atk * (attacker.atk_bonus.max(0) as u32 + 64);

    let eff_def = effective_level(defender.def_level, stance_def_bonus(defender.stance));
    let def_roll = eff_def * (defender.def_bonus.max(0) as u32 + 64);

    let chance = if atk_roll > def_roll {
        1.0 - (def_roll as f64 + 2.0) / (2.0 * (atk_roll as f64 + 1.0))
    } else {
        atk_roll as f64 / (2.0 * (def_roll as f64 + 1.0))
    };

    rand::rng().random::<f64>() < chance
}

pub fn roll_damage(max: u16) -> u16 {
    if max == 0 {
        return 0;
    }
    rand::rng().random_range(0..=max)
}

pub fn def_bonus_for_type(atk_type: AttackType, def_stab: i16, def_slash: i16, def_crush: i16) -> i16 {
    match atk_type {
        AttackType::Stab => def_stab,
        AttackType::Slash => def_slash,
        AttackType::Crush => def_crush,
        _ => 0,
    }
}

pub fn atk_bonus_for_type(atk_type: AttackType, atk_stab: i16, atk_slash: i16, atk_crush: i16) -> i16 {
    match atk_type {
        AttackType::Stab => atk_stab,
        AttackType::Slash => atk_slash,
        AttackType::Crush => atk_crush,
        _ => 0,
    }
}
