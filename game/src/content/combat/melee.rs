use super::{CombatTarget, PendingHit, anim, player, queue_hit, roll_hit, special};
use crate::player::Player;

pub async fn fire_melee_attack(player: &mut Player, target: CombatTarget) {
    let (atk, style) = player::melee_atk(player);
    let world = player.world();
    let def = target.def(&world, style.atk_type);
    drop(world);

    let attacker = CombatTarget::Player(player.index);

    if let Some(result) = special::try_execute(player, target, &atk, &def, style.atk_type) {
        special::apply_result(player, target, &result).await;
        player::award_combat_xp(style.xp_type, special::total_damage(&result)).await;
    } else {
        let anim = anim::attack(player);
        player.seq(anim);

        let (hit_type, damage) = roll_hit(&atk, &def, style.atk_type);
        let world = player.world();
        queue_hit(
            &world,
            PendingHit {
                target,
                attacker,
                damage,
                hit_type,
                delay: 0,
            },
        );
        player::award_combat_xp(style.xp_type, damage).await;
    }
}
