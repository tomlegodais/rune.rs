use filesystem::AttackType;

use crate::content::{
    CombatTarget, NpcAttackResult, NpcHit, Projectile, combat::ranged, npc_center, npc_melee_atk, player_def,
};

const RANGED_ANIM: u16 = 6953;
const PROJ_GFX: u16 = 1191;
const PROJ_START_CYCLE: u16 = 41;
const PROJ_SPEED: u16 = 20;

#[macros::npc_combat(npc_id = 6232)]
fn aviansie_attack() {
    let atk = npc_melee_atk(npc);
    let def = player_def(world, target, AttackType::Ranged);
    let (hit_type, damage) = crate::content::combat::roll_hit(&atk, &def, AttackType::Ranged);
    let src = npc_center(npc);
    let dst = match target {
        CombatTarget::Player(i) => world.player(i).position,
        CombatTarget::Npc(i) => world.npc(i).position,
    };

    let dist = ranged::distance_to_target(src, dst, 1);
    let delay = ranged::hit_delay(dist);

    NpcAttackResult {
        anim: RANGED_ANIM,
        hits: vec![NpcHit {
            target,
            damage,
            hit_type,
            delay,
            projectile: Some(Projectile {
                graphic_id: PROJ_GFX,
                src,
                dst,
                target,
                start_height: 40,
                end_height: 0,
                start_cycle: PROJ_START_CYCLE,
                end_cycle: PROJ_START_CYCLE + ranged::flight_time(PROJ_SPEED, dist),
                slope: 15,
                angle: 11,
            }),
        }],
    }
}
