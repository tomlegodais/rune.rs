use filesystem::AttackType;
use rand::Rng;

use crate::{
    content::{
        CombatTarget, NpcAttackResult, NpcHit, Projectile, accuracy, npc_center, npc_melee_atk, player_def,
        roll_npc_hit,
    },
    entity::HitType,
    npc::Npc,
};

const MELEE_SEQ: u16 = 7060;
const RANGED_SEQ: u16 = 7063;
const RANGED_SPOTANIM: u16 = 1200;

const SHOUTS: &[&str] = &[
    "Death to our enemies!",
    "Brargh!",
    "Break their bones!",
    "For the glory of Bandos!",
    "Split their skulls!",
    "We feast on the bones of our enemies tonight!",
    "CHAAARGE!",
    "Crush them underfoot!",
    "All glory to Bandos!",
    "GRAARDOR SMASH!",
    "For Bandos!",
];

fn maybe_shout(npc: &mut Npc) {
    let mut rng = rand::rng();
    if rng.random_ratio(1, 6) {
        npc.force_talk(SHOUTS[rng.random_range(0..SHOUTS.len())].to_string());
    }
}

#[macros::npc_combat(npc_id = 6260)]
fn graardor_attack() {
    maybe_shout(npc);
    if rand::rng().random_ratio(1, 3) {
        ranged_slam(npc, world)
    } else {
        NpcAttackResult {
            seq: MELEE_SEQ,
            hits: vec![roll_npc_hit(npc, target, world, AttackType::Crush, 0)],
        }
    }
}

fn ranged_slam(npc: &Npc, world: &crate::world::World) -> NpcAttackResult {
    let atk = npc_melee_atk(npc);
    let npc_pos = npc.position;
    let src = npc_center(npc);

    let hits = world
        .players
        .keys()
        .into_iter()
        .filter_map(|idx| {
            let player = world.player(idx);
            if player.position.plane != npc_pos.plane
                || (player.position.x - npc_pos.x).abs() > 15
                || (player.position.y - npc_pos.y).abs() > 15
            {
                return None;
            }

            let dst = player.position;
            drop(player);

            let target = CombatTarget::Player(idx);
            let def = player_def(world, target, AttackType::Crush);
            let hit = accuracy(&atk, &def, AttackType::Crush);
            let (hit_type, damage) =
                if hit { (HitType::Normal, rand::rng().random_range(15..=35)) } else { (HitType::Block, 0) };

            Some(NpcHit {
                target,
                damage,
                hit_type,
                delay: 1,
                projectile: Some(Projectile {
                    spotanim: RANGED_SPOTANIM,
                    src,
                    dst,
                    target,
                    start_height: 80,
                    end_height: 36,
                    start_cycle: 0,
                    end_cycle: 60,
                    slope: 16,
                    angle: 64,
                }),
            })
        })
        .collect();

    NpcAttackResult { seq: RANGED_SEQ, hits }
}
