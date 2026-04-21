use super::super::{
    formula::{accuracy, max_hit, roll_damage},
    special::SpecialResult,
};

#[macros::special_attack(obj_id = 14484, energy = 500)]
fn dclaws_spec() {
    let max = max_hit(atk);

    let (h1, h2, h3, h4) = if accuracy(atk, def, atk_type) {
        let h1 = roll_damage(max).max(1);
        let h2 = h1 / 2;
        let h3 = h2 / 2;
        (h1, h2, h3, h2 - h3)
    } else if accuracy(atk, def, atk_type) {
        let h2 = roll_damage(max).max(1);
        let h3 = h2 / 2;
        (0, h2, h3, h2 - h3)
    } else if accuracy(atk, def, atk_type) {
        let h3 = roll_damage(max * 3 / 4).max(1);
        (0, 0, h3, h3 + 1)
    } else {
        (0, 0, 0, roll_damage(max + max / 4).max(1))
    };

    SpecialResult {
        hits: vec![hit!(h1), hit!(h2), hit!(h3), hit!(h4)],
        seq: 10961,
        spot_anim: Some(1950),
        projectiles: vec![],
    }
}
