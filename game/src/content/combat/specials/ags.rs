use super::super::{
    formula::{accuracy, max_hit, roll_damage},
    special::{SpecialHit, SpecialResult},
};
use crate::{content::MeleeAttack, entity::HitType};

#[macros::special_attack(obj_id = 11694, energy = 500)]
fn ags_spec() {
    let boosted_atk = MeleeAttack {
        atk_bonus: atk.atk_bonus * 2,
        ..*atk
    };

    let base_max = max_hit(atk);
    let boosted_max = base_max + base_max * 375 / 1000;

    let hit = if accuracy(&boosted_atk, def, atk_type) {
        let dmg = roll_damage(boosted_max);
        SpecialHit {
            hit_type: if dmg == 0 { HitType::Block } else { HitType::Normal },
            damage: dmg,
        }
    } else {
        SpecialHit {
            hit_type: HitType::Block,
            damage: 0,
        }
    };

    SpecialResult {
        hits: vec![hit],
        anim: 7074,
        gfx: Some(1222),
    }
}
