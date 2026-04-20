use std::collections::HashMap;

use filesystem::{AttackType, WeaponStance};

use super::{
    CombatTarget, PendingHit, Projectile,
    formula::{AttackRoll, DefenceRoll, def_bonus_for_type},
    queue_hit, roll_hit,
};
use crate::{
    content::combat::ranged,
    npc::Npc,
    provider,
    world::{Position, World, can_interact_rect, find_path, has_line_of_sight},
};

const MAX_CHASE_DISTANCE: i32 = 16;
const HEAL_DELAY_TICKS: u16 = 50;

pub struct NpcHit {
    pub target: CombatTarget,
    pub damage: u16,
    pub hit_type: crate::entity::HitType,
    pub delay: u16,
    pub projectile: Option<Projectile>,
}

pub struct NpcAttackResult {
    pub anim: u16,
    pub hits: Vec<NpcHit>,
}

pub type NpcAttackFn = fn(&mut Npc, CombatTarget, &World) -> NpcAttackResult;

pub struct NpcCombatScript {
    pub npc_id: u16,
    pub attack: NpcAttackFn,
}

inventory::collect!(NpcCombatScript);

static SCRIPTS: std::sync::LazyLock<HashMap<u16, NpcAttackFn>> = std::sync::LazyLock::new(|| {
    inventory::iter::<NpcCombatScript>()
        .map(|e| (e.npc_id, e.attack))
        .collect()
});

fn get_attack_fn(npc_id: u16) -> Option<NpcAttackFn> {
    SCRIPTS.get(&npc_id).copied()
}

pub fn npc_size(npc_id: u16) -> i32 {
    provider::get_npc_type(npc_id as u32)
        .map(|t| t.size as i32)
        .unwrap_or(1)
}

pub fn npc_def(combat: &crate::npc::NpcCombat, atk_type: AttackType) -> DefenceRoll {
    DefenceRoll {
        def_level: combat.def_level,
        def_bonus: def_bonus_for_type(
            atk_type,
            combat.def_stab,
            combat.def_slash,
            combat.def_crush,
            combat.def_ranged,
            combat.def_magic,
        ),
        stance: WeaponStance::Defensive,
    }
}

pub fn npc_melee_atk(npc: &Npc) -> AttackRoll {
    let c = &npc.combat;
    AttackRoll {
        atk_level: c.atk_level,
        str_level: c.str_level,
        atk_bonus: c.atk_bonus,
        str_bonus: c.str_bonus,
        stance: WeaponStance::Accurate,
    }
}

pub fn player_def(world: &World, target: CombatTarget, atk_type: AttackType) -> DefenceRoll {
    let CombatTarget::Player(idx) = target else { unreachable!() };
    super::player::player_def(&world.player(idx), atk_type)
}

pub fn roll_npc_hit(npc: &Npc, target: CombatTarget, world: &World, atk_type: AttackType, delay: u16) -> NpcHit {
    let atk = npc_melee_atk(npc);
    let def = player_def(world, target, atk_type);
    let (hit_type, damage) = roll_hit(&atk, &def, atk_type);
    NpcHit {
        target,
        damage,
        hit_type,
        delay,
        projectile: None,
    }
}

pub fn npc_center(npc: &Npc) -> Position {
    let size = npc_size(npc.npc_id);
    Position::new(npc.position.x + size / 2, npc.position.y + size / 2, npc.position.plane)
}

pub fn is_adjacent(player_pos: Position, npc_pos: Position, size: i32) -> bool {
    can_interact_rect(provider::get_collision(), player_pos, npc_pos, size, size, 0)
}

pub fn follow_target(npc: &mut Npc, target_pos: Position, size: i32) {
    let (px, py) = (target_pos.x, target_pos.y);
    let plane = npc.position.plane;

    let mut candidates = Vec::with_capacity((size as usize) * 4);

    for dy in (-size + 1)..=0 {
        candidates.push(Position::new(px + 1, py + dy, plane));
    }
    for dy in (-size + 1)..=0 {
        candidates.push(Position::new(px - size, py + dy, plane));
    }
    for dx in (-size + 1)..=0 {
        candidates.push(Position::new(px + dx, py + 1, plane));
    }
    for dx in (-size + 1)..=0 {
        candidates.push(Position::new(px + dx, py - size, plane));
    }

    let best = candidates
        .into_iter()
        .map(|dest| find_path(npc.position, dest))
        .filter(|path| !path.is_empty())
        .min_by_key(|path| path.len());

    if let Some(path) = best {
        npc.entity.walk_queue = path;
    }
}

fn default_attack(npc: &mut Npc, target: CombatTarget, world: &World) -> NpcAttackResult {
    NpcAttackResult {
        anim: npc.combat.atk_seq,
        hits: vec![roll_npc_hit(npc, target, world, AttackType::Crush, 0)],
    }
}

#[macros::npc_action]
pub async fn start_combat(target_index: usize) {
    let player_face = target_index as u16 + 32768;
    npc.entity.face_target = Some(player_face);
    npc.masks.add(crate::npc::FaceEntityMask(player_face));
    npc.combat_target = Some(target_index);

    let atk_speed = npc.combat.atk_speed;
    let atk_range = npc.combat.atk_range;
    let spawn_pos = npc.spawn_position;
    let attack_fn = get_attack_fn(npc.npc_id);
    let mut cd: u16 = (atk_speed / 2).saturating_sub(1);

    delay!(1);

    loop {
        let world = npc.entity.world();
        if !world.players.contains(target_index) || world.player(target_index).hitpoints().is_dying() {
            break;
        }

        let target_pos = world.player(target_index).position;
        let size = npc_size(npc.npc_id);
        drop(world);

        let from_spawn = (npc.position.x - spawn_pos.x)
            .abs()
            .max((npc.position.y - spawn_pos.y).abs());
        let to_target = (npc.position.x - target_pos.x)
            .abs()
            .max((npc.position.y - target_pos.y).abs());
        if from_spawn > MAX_CHASE_DISTANCE || to_target > MAX_CHASE_DISTANCE || npc.position.plane != target_pos.plane {
            break;
        }

        npc.masks.add(crate::npc::FaceEntityMask(player_face));

        let in_range = if atk_range > 0 {
            let dist = ranged::distance_to_target(npc.position, target_pos, 1);
            let collision = provider::get_collision();
            dist <= atk_range as i32
                && npc.position.plane == target_pos.plane
                && has_line_of_sight(collision, npc_center(&npc), target_pos)
        } else {
            is_adjacent(target_pos, npc.position, size)
        };

        if in_range && cd == 0 {
            npc.entity.stop();

            let world = npc.entity.world();
            let target = CombatTarget::Player(target_index);
            let result = match attack_fn {
                Some(f) => f(&mut npc, target, &world),
                None => default_attack(&mut npc, target, &world),
            };

            npc.seq(result.anim);
            let attacker = CombatTarget::Npc(npc.index);
            for hit in result.hits {
                if let Some(proj) = &hit.projectile {
                    super::broadcast_projectile(&world, proj).await;
                }
                queue_hit(
                    &world,
                    PendingHit {
                        target: hit.target,
                        attacker,
                        damage: hit.damage,
                        hit_type: hit.hit_type,
                        delay: hit.delay,
                    },
                );
            }

            cd = atk_speed;
        } else if !in_range {
            follow_target(&mut npc, target_pos, size);
        }

        cd = cd.saturating_sub(1);
        delay!(1);
    }

    npc.combat_target = None;
    npc.entity.face_target = None;
    npc.masks.add(crate::npc::FaceEntityMask(65535));

    if npc.position != spawn_pos {
        npc.entity.walk_queue = find_path(npc.position, spawn_pos);
    }

    delay!(HEAL_DELAY_TICKS);
    npc.current_hp = npc.max_hp;
}
