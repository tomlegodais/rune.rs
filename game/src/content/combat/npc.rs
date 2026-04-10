use filesystem::{AttackType, WeaponStance};

use super::{
    CombatTarget, PendingHit,
    formula::{MeleeAttack, MeleeDefence, def_bonus_for_type},
    queue_hit, roll_hit,
};
use crate::{
    provider,
    world::{Position, can_interact_rect, find_path},
};

const MAX_CHASE_DISTANCE: i32 = 16;
const HEAL_DELAY_TICKS: u16 = 50;

pub fn npc_size(npc_id: u16) -> i32 {
    provider::get_npc_type(npc_id as u32)
        .map(|t| t.size as i32)
        .unwrap_or(1)
}

pub fn melee_def(combat: &crate::npc::NpcCombat, atk_type: AttackType) -> MeleeDefence {
    MeleeDefence {
        def_level: combat.def_level,
        def_bonus: def_bonus_for_type(atk_type, combat.def_stab, combat.def_slash, combat.def_crush),
        stance: WeaponStance::Defensive,
    }
}

fn is_adjacent(player_pos: Position, npc_pos: Position, size: i32) -> bool {
    can_interact_rect(provider::get_collision(), player_pos, npc_pos, size, size, 0)
}

fn follow_target(npc: &mut crate::npc::Npc, target_pos: Position, size: i32) {
    let (px, py) = (target_pos.x, target_pos.y);
    let plane = npc.position.plane;

    let mut candidates = Vec::with_capacity((size as usize) * 4);

    // west edge: npc_x = px + 1, npc_y in [py - size + 1 .. py]
    for dy in (-size + 1)..=0 {
        candidates.push(Position::new(px + 1, py + dy, plane));
    }
    // east edge: npc_x = px - size, npc_y in [py - size + 1 .. py]
    for dy in (-size + 1)..=0 {
        candidates.push(Position::new(px - size, py + dy, plane));
    }
    // north edge: npc_y = py + 1, npc_x in [px - size + 1 .. px]
    for dx in (-size + 1)..=0 {
        candidates.push(Position::new(px + dx, py + 1, plane));
    }
    // south edge: npc_y = py - size, npc_x in [px - size + 1 .. px]
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

#[macros::npc_action]
pub async fn start_combat(target_index: usize) {
    let player_face = target_index as u16 + 32768;
    npc.entity.face_target = Some(player_face);
    npc.masks.add(crate::npc::FaceEntityMask(player_face));
    npc.combat_target = Some(target_index);

    let atk_speed = npc.combat.atk_speed;
    let spawn_pos = npc.spawn_position;
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

        if is_adjacent(target_pos, npc.position, size) && cd == 0 {
            npc.entity.stop();

            let world = npc.entity.world();
            let combat = &npc.combat;
            let atk = MeleeAttack {
                atk_level: combat.atk_level,
                str_level: combat.str_level,
                atk_bonus: combat.atk_bonus,
                str_bonus: combat.str_bonus,
                stance: WeaponStance::Accurate,
            };
            let atk_seq = combat.atk_seq;

            let def = super::player::melee_def(&world.player(target_index), AttackType::Crush);
            let (hit_type, damage) = roll_hit(&atk, &def, AttackType::Crush);
            drop(world);

            npc.seq(atk_seq);
            queue_hit(
                &npc.entity.world(),
                PendingHit {
                    target: CombatTarget::Player(target_index),
                    attacker: CombatTarget::Npc(npc.index),
                    damage,
                    hit_type,
                    delay: 0,
                },
            );

            cd = atk_speed;
        } else if !is_adjacent(target_pos, npc.position, size) {
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
