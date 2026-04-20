use super::super::{
    formula::{accuracy, max_hit, roll_damage},
    special::SpecialResult,
};

#[macros::special_attack(obj_id = 4153, energy = 500, instant = true)]
fn gmaul_spec() {
    let dmg = if accuracy(atk, def, atk_type) { roll_damage(max_hit(atk)) } else { 0 };

    SpecialResult {
        hits: vec![hit!(dmg)],
        anim: 1667,
        gfx: Some(340),
        projectiles: vec![],
    }
}
