use crate::{
    content::combat::{
        formula::{MeleeAttack, accuracy, max_hit, roll_damage},
        special::{SpecialHit, SpecialResult},
    },
    entity::HitType,
};

#[macros::special_attack(obj_id = 5698, energy = 250)]
fn dds_spec() {
    let boosted_atk = MeleeAttack {
        atk_bonus: atk.atk_bonus + atk.atk_bonus * 15 / 100,
        ..*atk
    };

    let base_max = max_hit(atk);
    let boosted_max = base_max + base_max * 15 / 100;

    let hits: Vec<SpecialHit> = (0..2)
        .map(|_| {
            if accuracy(&boosted_atk, def, atk_type) {
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
            }
        })
        .collect();

    SpecialResult {
        hits,
        anim: 1062,
        gfx: Some(252),
    }
}
