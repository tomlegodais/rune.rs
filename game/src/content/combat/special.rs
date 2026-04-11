use std::collections::HashMap;

use super::{
    CombatTarget, PendingHit,
    formula::{MeleeAttack, MeleeDefence},
    queue_hit,
};
use crate::{entity::HitType, player::Player};

pub type SpecialExecuteFn =
    fn(&mut Player, &MeleeAttack, &MeleeDefence, filesystem::AttackType, CombatTarget) -> SpecialResult;

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
    pub instant: bool,
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

pub fn is_instant(player: &Player) -> bool {
    player
        .worn()
        .slot(filesystem::WearPos::Weapon)
        .and_then(|obj| get(obj.id))
        .is_some_and(|e| e.instant)
}

pub fn try_execute(
    player: &mut Player,
    target: CombatTarget,
    atk: &MeleeAttack,
    def: &MeleeDefence,
    atk_type: filesystem::AttackType,
) -> Option<SpecialResult> {
    let weapon = player.worn().slot(filesystem::WearPos::Weapon)?;
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
    for (i, hit) in result.hits.iter().enumerate() {
        queue_hit(
            &world,
            PendingHit {
                target,
                attacker,
                damage: hit.damage,
                hit_type: hit.hit_type,
                delay: (i / 2) as u16,
            },
        );
    }
}

pub fn total_damage(result: &SpecialResult) -> u16 {
    result.hits.iter().map(|h| h.damage).sum()
}
