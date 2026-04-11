use super::super::{
    formula::{accuracy, max_hit, roll_damage},
    special::SpecialResult,
};

#[macros::special_attack(obj_id = 11694, energy = 500)]
fn ags_spec() {
    let boosted_atk = crate::content::MeleeAttack {
        atk_bonus: atk.atk_bonus * 2,
        ..*atk
    };

    let base_max = max_hit(atk);
    let boosted_max = base_max + base_max * 375 / 1000;

    let dmg = if accuracy(&boosted_atk, def, atk_type) { roll_damage(boosted_max) } else { 0 };

    SpecialResult {
        hits: vec![hit!(dmg)],
        anim: 7074,
        gfx: Some(1222),
    }
}
