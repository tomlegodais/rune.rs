use super::super::{
    Projectile,
    formula::{accuracy, max_hit, roll_damage},
    ranged,
    special::SpecialResult,
};

const FIRST_ARROW_SPOTANIM: u16 = 1099;
const SECOND_ARROW_SPOTANIM: u16 = 1099;
const MIN_HIT: u16 = 5;
const DRAGON_ARROW: u16 = 11212;
const DRAGON_MIN_HIT: u16 = 8;

#[macros::special_attack(obj_id = 11235, energy = 550)]
fn darkbow_spec() {
    let base_max = max_hit(atk);
    let boosted_max = base_max + base_max * 30 / 100;
    let ammo_id = player.worn().slot(filesystem::WearPos::Ammo).map(|o| o.id).unwrap_or(0);
    let min = if ammo_id == DRAGON_ARROW { DRAGON_MIN_HIT } else { MIN_HIT };
    let world = player.world();
    let target_pos = target.position(&world);
    let target_size = target.size(&world);
    let dist = ranged::distance_to_target(player.position, target_pos, target_size);
    let delay = ranged::hit_delay(dist);
    drop(world);

    let roll = |min: u16| -> u16 { if accuracy(atk, def, atk_type) { roll_damage(boosted_max).max(min) } else { 0 } };
    let make_proj = |spotanim: u16, start: u16, angle: u8| Projectile {
        spotanim,
        src: player.position,
        dst: target_pos,
        target,
        start_height: 40,
        end_height: 36,
        start_cycle: start,
        end_cycle: start + (dist as u16) * 5,
        slope: 21,
        angle,
    };

    SpecialResult {
        hits: vec![hit!(roll(min), delay), hit!(roll(min), delay + 1)],
        seq: 426,
        spot_anim: None,
        projectiles: vec![
            make_proj(FIRST_ARROW_SPOTANIM, 41, 5),
            make_proj(SECOND_ARROW_SPOTANIM, 51, 11),
        ],
    }
}
