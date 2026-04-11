use super::super::{
    formula::{accuracy, max_hit, roll_damage},
    special::SpecialResult,
};

#[macros::special_attack(obj_id = 5698, energy = 250)]
fn dds_spec() {
    let boosted_atk = crate::content::MeleeAttack {
        atk_bonus: atk.atk_bonus + atk.atk_bonus * 15 / 100,
        ..*atk
    };

    let base_max = max_hit(atk);
    let boosted_max = base_max + base_max * 15 / 100;

    let hits = (0..2)
        .map(|_| {
            let dmg = if accuracy(&boosted_atk, def, atk_type) { roll_damage(boosted_max) } else { 0 };
            hit!(dmg)
        })
        .collect();

    SpecialResult {
        hits,
        anim: 1062,
        gfx: Some(252),
    }
}
