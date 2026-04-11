mod anim;
mod formula;
mod npc;
mod npcs;
mod player;
mod special;
mod specials;

pub use formula::{MeleeAttack, MeleeDefence, accuracy, max_hit, roll_damage};
pub use npc::{
    NpcAttackResult, NpcCombatScript, NpcHit, npc_center, npc_melee_atk, npc_size, player_def, roll_npc_hit,
};
pub use player::melee_atk;
pub use special::get as get_spec;

use crate::{
    entity::{Hit, HitType},
    player::{Clientbound, FaceEntityMask as PlayerFaceEntityMask},
    provider,
    world::{Position, World, can_interact_rect, find_path_adjacent_rect},
};

pub struct PendingHit {
    pub target: CombatTarget,
    pub attacker: CombatTarget,
    pub damage: u16,
    pub hit_type: HitType,
    pub delay: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CombatTarget {
    Npc(usize),
    Player(usize),
}

impl CombatTarget {
    fn client_index(self) -> u16 {
        match self {
            Self::Npc(i) => i as u16,
            Self::Player(i) => i as u16 + 32768,
        }
    }

    fn alive(self, world: &World) -> bool {
        match self {
            Self::Npc(i) => world.npcs.contains(i) && !world.npc(i).is_dying(),
            Self::Player(i) => world.players.contains(i) && !world.player(i).hitpoints().is_dying(),
        }
    }

    fn position(self, world: &World) -> crate::world::Position {
        match self {
            Self::Npc(i) => world.npc(i).position,
            Self::Player(i) => world.player(i).position,
        }
    }

    fn size(self, world: &World) -> i32 {
        match self {
            Self::Npc(i) => npc::npc_size(world.npc(i).npc_id),
            Self::Player(_) => 1,
        }
    }

    fn melee_def(self, world: &World, atk_type: filesystem::AttackType) -> MeleeDefence {
        match self {
            Self::Npc(i) => npc::melee_def(&world.npc(i).combat, atk_type),
            Self::Player(i) => player::melee_def(&world.player(i), atk_type),
        }
    }

    fn apply_hit(self, world: &World, damage: u16, hit_type: HitType, attacker: CombatTarget) {
        match self {
            Self::Npc(i) => apply_hit_npc(world, i, damage, hit_type),
            Self::Player(i) => apply_hit_player(world, i, damage, hit_type, attacker),
        }
    }
}

pub fn queue_hit(world: &World, hit: PendingHit) {
    world.pending_hits.lock().push(hit);
}

pub struct Projectile {
    pub graphic_id: u16,
    pub src: Position,
    pub dst: Position,
    pub target: CombatTarget,
    pub start_height: u8,
    pub end_height: u8,
    pub start_cycle: u16,
    pub end_cycle: u16,
    pub slope: u8,
    pub angle: u8,
}

fn proj_target_index(target: CombatTarget) -> i16 {
    match target {
        CombatTarget::Npc(i) => (i + 1) as i16,
        CombatTarget::Player(i) => -((i + 1) as i16),
    }
}

fn build_projanim(proj: &Projectile, region_base: Position) -> net::MapProjAnim {
    let (zone_x, zone_y, _) = proj.src.zone_coords(region_base);
    net::MapProjAnim {
        zone_frame: net::ZoneFrame::new(zone_x, zone_y, proj.src.plane as u8),
        packed_pos: (((proj.src.x & 7) << 3) | (proj.src.y & 7)) as u8,
        dst_dx: (proj.dst.x - proj.src.x) as i8,
        dst_dy: (proj.dst.y - proj.src.y) as i8,
        target: proj_target_index(proj.target),
        graphic_id: proj.graphic_id,
        start_height: proj.start_height,
        end_height: proj.end_height,
        start_cycle: proj.start_cycle,
        end_cycle: proj.end_cycle,
        slope: proj.slope,
        angle: proj.angle,
    }
}

pub async fn send_projectile(player: &mut crate::player::Player, world: &World, proj: &Projectile) {
    player
        .map_projanim(build_projanim(proj, player.viewport.region_base))
        .await;

    for index in world.players.keys() {
        if index == player.index {
            continue;
        }
        let mut other = world.players.get_mut(index);
        if !other.viewport.is_within_view(other.position, proj.src) {
            continue;
        }
        let frame = build_projanim(proj, other.viewport.region_base);
        other.map_projanim(frame).await;
    }
}

pub async fn broadcast_projectile(world: &World, proj: &Projectile) {
    for index in world.players.keys() {
        let mut player = world.players.get_mut(index);
        if !player.viewport.is_within_view(player.position, proj.src) {
            continue;
        }
        let frame = build_projanim(proj, player.viewport.region_base);
        player.map_projanim(frame).await;
    }
}

pub fn process_pending_hits(world: &World) {
    let ready: Vec<PendingHit> = {
        let mut queue = world.pending_hits.lock();
        let (ready, mut pending): (Vec<_>, Vec<_>) = queue.drain(..).partition(|h| h.delay == 0);
        pending.iter_mut().for_each(|h| h.delay -= 1);
        *queue = pending;
        ready
    };

    for hit in ready {
        if !hit.target.alive(world) {
            continue;
        }
        hit.target.apply_hit(world, hit.damage, hit.hit_type, hit.attacker);
    }
}

pub fn roll_hit(atk: &MeleeAttack, def: &MeleeDefence, atk_type: filesystem::AttackType) -> (HitType, u16) {
    if accuracy(atk, def, atk_type) {
        let max = max_hit(atk);
        let dmg = roll_damage(max);
        (if dmg == 0 { HitType::Block } else { HitType::Normal }, dmg)
    } else {
        (HitType::Block, 0)
    }
}

fn apply_hit_npc(world: &World, npc_index: usize, damage: u16, hit_type: HitType) {
    world.npc_mut(npc_index).damage(Hit::new(damage, hit_type));

    let npc = world.npc(npc_index);
    let play_block = !npc.is_dying() && !npc.has_seq();
    let block_anim = npc.combat.block_seq;
    drop(npc);
    if play_block {
        world.npc_mut(npc_index).seq(block_anim);
    }
}

fn apply_hit_player(world: &World, target_index: usize, damage: u16, hit_type: HitType, attacker: CombatTarget) {
    let target = world.player(target_index);
    let blk_anim = anim::block(&target);
    let has_seq = target.has_seq();
    drop(target);

    let died = world
        .player_mut(target_index)
        .hitpoints_mut()
        .damage(Hit::new(damage, hit_type));
    if !died {
        if !has_seq {
            world.player_mut(target_index).seq(blk_anim);
        }
        world.player_mut(target_index).combat_mut().queue_retaliate(attacker);
    }
}

#[macros::player_action]
pub async fn start_melee_combat(target: CombatTarget) {
    player.combat_mut().set_combat_target(Some(target));

    let target_face = target.client_index();
    player.entity.face_target = Some(target_face);
    player.player_info.add_mask(PlayerFaceEntityMask(target_face));

    if let CombatTarget::Npc(npc_index) = target {
        let world = player.world();
        if !world.npcs.contains(npc_index) {
            return;
        }
        crate::npc::fire_action(&mut world.npc_mut(npc_index), npc::start_combat(player.index));
    }

    let mut player_cd: u16 = if let CombatTarget::Player(target_index) = target {
        let world = player.world();
        let target_fighting_us = world.players.contains(target_index)
            && world.player(target_index).combat().combat_target() == Some(CombatTarget::Player(player.index));
        if target_fighting_us { 1 } else { 0 }
    } else {
        0
    };

    loop {
        let world = player.world();
        if !target.alive(&world) {
            break;
        }

        let target_pos = target.position(&world);
        let target_size = target.size(&world);
        let collision = provider::get_collision();

        if player.combat_mut().consume_eat_delay() {
            player_cd = player_cd.max(1);
        }

        if let CombatTarget::Player(ti) = target
            && player_cd == 0
            && player.world().players.contains(ti)
            && player.world().player(ti).has_seq()
        {
            player_cd = 1;
        }

        let adjacent = can_interact_rect(collision, player.position, target_pos, target_size, target_size, 0);

        if adjacent && player_cd > 0 && player.combat().spec_enabled() && special::is_instant(&player) {
            let (atk, style) = player::melee_atk(&player);
            let def = target.melee_def(&player.world(), style.atk_type);
            if let Some(result) = special::try_execute(&mut player, target, &atk, &def, style.atk_type) {
                special::apply_result(&mut player, target, &result);
                player::award_melee_xp(style.xp_type, special::total_damage(&result)).await;
            }
        } else if !adjacent {
            player.entity.walk_queue =
                find_path_adjacent_rect(player.position, target_pos, target_size, target_size, 0);
        } else if player_cd == 0 {
            player.entity.stop();

            let (atk, style) = player::melee_atk(&player);
            let def = target.melee_def(&player.world(), style.atk_type);

            let attacker = CombatTarget::Player(player.index);
            if let Some(result) = special::try_execute(&mut player, target, &atk, &def, style.atk_type) {
                special::apply_result(&mut player, target, &result);
                player::award_melee_xp(style.xp_type, special::total_damage(&result)).await;
            } else {
                let anim = anim::attack(&player);
                player.seq(anim);

                let (hit_type, damage) = roll_hit(&atk, &def, style.atk_type);
                queue_hit(
                    &player.world(),
                    PendingHit {
                        target,
                        attacker,
                        damage,
                        hit_type,
                        delay: 0,
                    },
                );
                player::award_melee_xp(style.xp_type, damage).await;
            }

            player_cd = player::weapon_atk_speed(&player);
        }

        player_cd = player_cd.saturating_sub(1);
        delay!(1);
    }

    player.entity.face_target = None;
    player.player_info.add_mask(PlayerFaceEntityMask(65535));
    player.combat_mut().set_combat_target(None);
}
