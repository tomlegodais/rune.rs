use filesystem::{WeaponCategory, WearPos};

use super::{CombatTarget, PendingHit, Projectile, queue_hit, roll_hit, send_projectile, special};
use crate::{
    player::{Obj, Player},
    provider,
    world::Position,
};

const DEFAULT_ATTACK_RANGE: i32 = 7;
const LONGRANGE_BONUS: i32 = 2;
const MAX_ATTACK_RANGE: i32 = 10;

const DEFAULT_PROJ_SPOTANIM: u16 = 15;

#[derive(Clone, Copy)]
struct ProjectileTemplate {
    start_height: u8,
    end_height: u8,
    delay: u16,
    curve: u8,
}

fn eucl_dist(src: Position, dst: Position) -> i32 {
    let dx = (dst.x - src.x) as f64;
    let dy = (dst.y - src.y) as f64;
    (dx * dx + dy * dy).sqrt() as i32
}

pub fn flight_time(speed: u16, dist: i32) -> u16 {
    let divisor = (speed / 10).max(1);
    dist as u16 * 30 / divisor
}

fn center_of(pos: Position, size: i32) -> Position {
    Position::new(pos.x + size / 2, pos.y + size / 2, pos.plane)
}

const THROWN: ProjectileTemplate = ProjectileTemplate {
    start_height: 43,
    end_height: 34,
    delay: 31,
    curve: 6,
};

const BOW_LIKE: ProjectileTemplate = ProjectileTemplate {
    start_height: 43,
    end_height: 34,
    delay: 41,
    curve: 6,
};

const THROWN_SPEED: u16 = 42;

fn proj_template(player: &Player) -> ProjectileTemplate {
    match player.worn().weapon_category() {
        Some(WeaponCategory::Thrown | WeaponCategory::Chinchompa) => THROWN,
        _ => BOW_LIKE,
    }
}

fn proj_start_offset(dist: i32) -> u8 {
    match dist {
        d if d > 2 => 0,
        _ => 11,
    }
}

fn eff_speed(cat: Option<WeaponCategory>, dist: i32) -> u16 {
    match cat {
        Some(WeaponCategory::Thrown | WeaponCategory::Chinchompa) => THROWN_SPEED,
        Some(WeaponCategory::Crossbow) => match dist {
            d if d > 2 => 61,
            _ => 51,
        },
        _ => match dist {
            d if d < 2 => 51,
            2 => 56,
            _ => 61,
        },
    }
}

pub fn attack_range(player: &Player) -> i32 {
    let style = super::player::resolve_style(player);
    let longrange = style.stance == filesystem::WeaponStance::Longrange;
    let base = player
        .worn()
        .slot(WearPos::Weapon)
        .and_then(|obj| provider::get_obj_type(obj.id as u32))
        .and_then(|t| t.atk_range)
        .unwrap_or(DEFAULT_ATTACK_RANGE as i16) as i32;
    (base + if longrange { LONGRANGE_BONUS } else { 0 }).min(MAX_ATTACK_RANGE)
}

pub fn nearest_tile(pos: Position, target: Position, size: i32) -> Position {
    let nx = pos.x.clamp(target.x, target.x + size - 1);
    let ny = pos.y.clamp(target.y, target.y + size - 1);
    Position::new(nx, ny, target.plane)
}

pub fn distance_to_target(pos: Position, target: Position, size: i32) -> i32 {
    let dx = if pos.x < target.x {
        target.x - pos.x
    } else if pos.x >= target.x + size {
        pos.x - (target.x + size - 1)
    } else {
        0
    };
    let dy = if pos.y < target.y {
        target.y - pos.y
    } else if pos.y >= target.y + size {
        pos.y - (target.y + size - 1)
    } else {
        0
    };
    dx.max(dy)
}

fn proj_spotanim(player: &Player) -> u16 {
    ranged_source(player)
        .and_then(|t| t.proj_spotanim)
        .unwrap_or(DEFAULT_PROJ_SPOTANIM)
}

fn attack_spot_anim(player: &Player) -> Option<u16> {
    ranged_source(player).and_then(|t| t.atk_spotanim)
}

fn ranged_source(player: &Player) -> Option<&'static filesystem::ObjType> {
    let wearpos = match player.worn().weapon_category() {
        Some(WeaponCategory::Thrown | WeaponCategory::Chinchompa) => WearPos::Weapon,
        _ => WearPos::Ammo,
    };
    player
        .worn()
        .slot(wearpos)
        .and_then(|obj| provider::get_obj_type(obj.id as u32))
}

pub fn hit_delay(distance: i32) -> u16 {
    (1 + (1 + distance) / 3).max(1) as u16
}

fn uses_ammo_slot(cat: WeaponCategory) -> bool {
    matches!(cat, WeaponCategory::Bow | WeaponCategory::Crossbow)
}

fn is_thrown(cat: WeaponCategory) -> bool {
    matches!(cat, WeaponCategory::Thrown | WeaponCategory::Chinchompa)
}

fn consume_ammo(player: &mut Player, target_pos: Position) {
    let cat = player.worn().weapon_category().unwrap_or(WeaponCategory::Unarmed);
    let ammo_id = if uses_ammo_slot(cat) {
        let Some(ammo) = player.worn().slot(WearPos::Ammo) else { return };
        let new = (ammo.amount > 1).then(|| Obj::new(ammo.id, ammo.amount - 1));
        player.worn_mut().set(WearPos::Ammo, new);
        ammo.id
    } else if is_thrown(cat) {
        let Some(weapon) = player.worn().slot(WearPos::Weapon) else { return };
        let new = (weapon.amount > 1).then(|| Obj::new(weapon.id, weapon.amount - 1));
        player.worn_mut().set(WearPos::Weapon, new);
        weapon.id
    } else {
        return;
    };

    let world = player.world();
    world.obj_stacks.add(ammo_id, 1, target_pos, Some(player.index));
}

fn ammo_compatible(player: &Player) -> bool {
    let weapon_type = player
        .worn()
        .slot(WearPos::Weapon)
        .and_then(|obj| provider::get_obj_type(obj.id as u32));

    let Some(weapon) = weapon_type else { return true };
    let Some(required_type) = weapon.ammo_type else { return true };
    let weapon_tier = weapon.ammo_tier.unwrap_or(0);
    let ammo_type = player
        .worn()
        .slot(WearPos::Ammo)
        .and_then(|obj| provider::get_obj_type(obj.id as u32));

    let Some(ammo) = ammo_type else { return false };
    let Some(ammo_type) = ammo.ammo_type else { return false };
    let ammo_tier = ammo.ammo_tier.unwrap_or(0);

    required_type == ammo_type && ammo_tier <= weapon_tier
}

pub fn has_ammo(player: &Player) -> bool {
    let cat = player.worn().weapon_category().unwrap_or(WeaponCategory::Unarmed);
    if uses_ammo_slot(cat) {
        player.worn().slot(WearPos::Ammo).is_some() && ammo_compatible(player)
    } else if is_thrown(cat) {
        player.worn().slot(WearPos::Weapon).is_some()
    } else {
        true
    }
}

pub async fn fire_ranged_attack(player: &mut Player, target: CombatTarget) -> Option<u16> {
    let (atk, style) = super::player::ranged_atk(player);
    let world = player.world();
    let def = target.def(&world, style.atk_type);
    drop(world);

    if let Some(result) = special::try_execute(player, target, &atk, &def, style.atk_type) {
        let world = player.world();
        let target_pos = target.position(&world);
        drop(world);

        special::apply_result(player, target, &result).await;
        consume_ammo(player, target_pos);
        player.worn_mut().flush().await;

        let total = special::total_damage(&result);
        super::player::award_combat_xp(style.xp_type, total).await;
        return Some(total);
    }

    let (hit_type, damage) = roll_hit(&atk, &def, style.atk_type);

    let spotanim = proj_spotanim(player);
    let world = player.world();
    let target_pos = target.position(&world);
    let target_size = target.size(&world);
    let dist = distance_to_target(player.position, target_pos, target_size);
    drop(world);

    let src = player.position;
    let dst = center_of(target_pos, target_size);
    let tpl = proj_template(player);
    let cat = player.worn().weapon_category();
    let eucl = eucl_dist(src, target_pos);
    let speed = eff_speed(cat, eucl);
    let proj = Projectile {
        spotanim,
        src,
        dst,
        target,
        start_height: tpl.start_height,
        end_height: tpl.end_height,
        start_cycle: tpl.delay,
        end_cycle: tpl.delay + flight_time(speed, eucl),
        slope: tpl.curve,
        angle: proj_start_offset(eucl),
    };

    if let Some(id) = attack_spot_anim(player) {
        drop(player.spot_anim(id).speed(0).height(100));
    }

    let world = player.world();
    send_projectile(player, &world, &proj).await;

    let attack_seq = super::seq::attack(player);
    player.seq(attack_seq);

    consume_ammo(player, target_pos);
    player.worn_mut().flush().await;

    let attacker = CombatTarget::Player(player.index);
    let world = player.world();
    queue_hit(
        &world,
        PendingHit {
            target,
            attacker,
            damage,
            hit_type,
            delay: hit_delay(dist),
        },
    );

    super::player::award_combat_xp(style.xp_type, damage).await;
    Some(damage)
}
