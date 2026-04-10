use std::collections::HashMap;

use super::{
    CombatTarget,
    formula::{MeleeAttack, MeleeDefence},
};
use crate::{entity::HitType, player::Player};

pub type SpecialExecuteFn =
    fn(&mut Player, &MeleeAttack, &MeleeDefence, filesystem::config::AttackType, CombatTarget) -> SpecialResult;

pub struct SpecialHit {
    pub hit_type: HitType,
    pub damage: u16,
}

pub struct SpecialResult {
    pub hits: Vec<SpecialHit>,
    pub anim: u16,
    pub gfx: Option<u16>,
}

pub struct SpecialAttackEntry {
    pub obj_id: u16,
    pub energy_cost: u16,
    pub execute: SpecialExecuteFn,
}

inventory::collect!(SpecialAttackEntry);

static SPECIALS: std::sync::LazyLock<HashMap<u16, &'static SpecialAttackEntry>> =
    std::sync::LazyLock::new(|| inventory::iter::<SpecialAttackEntry>().map(|e| (e.obj_id, e)).collect());

pub fn get(obj_id: u16) -> Option<&'static SpecialAttackEntry> {
    SPECIALS.get(&obj_id).copied()
}

pub fn has_special(obj_id: u16) -> bool {
    SPECIALS.contains_key(&obj_id)
}

pub fn try_execute(
    player: &mut Player,
    target: CombatTarget,
    atk: &MeleeAttack,
    def: &MeleeDefence,
    atk_type: filesystem::config::AttackType,
) -> Option<SpecialResult> {
    let weapon = player.worn().slot(filesystem::config::WearPos::Weapon)?;
    let entry = get(weapon.id)?;

    if !player.combat().spec_enabled() {
        return None;
    }

    if player.combat().spec_energy() < entry.energy_cost {
        player.combat_mut().set_spec_enabled(false);
        return None;
    }

    player.combat_mut().drain_spec(entry.energy_cost);
    player.combat_mut().set_spec_enabled(false);

    Some((entry.execute)(player, atk, def, atk_type, target))
}

pub fn apply_result(player: &mut Player, target: CombatTarget, result: &SpecialResult) {
    player.seq(result.anim);
    if let Some(gfx) = result.gfx {
        player.spot_anim(gfx);
    }

    let world = player.world();
    let attacker = CombatTarget::Player(player.index);
    for hit in &result.hits {
        target.apply_hit(&world, hit.damage, hit.hit_type, attacker);
    }
}

pub fn total_damage(result: &SpecialResult) -> u16 {
    result.hits.iter().map(|h| h.damage).sum()
}
