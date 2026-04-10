use macros::command;

use super::CommandEntry;
use crate::{
    content::{CombatTarget, Projectile, max_hit, melee_atk, npc_size, send_projectile},
    player::Player,
    send_message,
    world::Position,
};

#[command(name = "maxhit")]
async fn maxhit(player: &mut Player) {
    let (atk, style) = melee_atk(player);
    let max = max_hit(&atk);
    send_message!(
        player,
        "Max hit: {} (style: {:?}, atk bonus: {}, str bonus: {})",
        max,
        style.atk_type,
        atk.atk_bonus,
        atk.str_bonus
    );
}

#[command(name = "proj")]
async fn proj(player: &mut Player, graphic: Option<u16>) {
    let gfx = graphic.unwrap_or(19);
    let src = player.position;
    let world = player.world();

    let nearest = world
        .npcs
        .keys()
        .into_iter()
        .filter(|&i| {
            let npc = world.npc(i);
            npc.position.plane == src.plane
                && (npc.position.x - src.x).abs() <= 15
                && (npc.position.y - src.y).abs() <= 15
        })
        .min_by_key(|&i| {
            let npc = world.npc(i);
            (npc.position.x - src.x).abs() + (npc.position.y - src.y).abs()
        });

    let Some(npc_index) = nearest else {
        send_message!(player, "No nearby NPC found.");
        return;
    };

    let npc = world.npc(npc_index);
    let size = npc_size(npc.npc_id);
    let dst = Position::new(npc.position.x + size / 2, npc.position.y + size / 2, npc.position.plane);
    drop(npc);
    drop(world);

    let proj = Projectile {
        graphic_id: gfx,
        src,
        dst,
        target: CombatTarget::Npc(npc_index),
        start_height: 40,
        end_height: 36,
        start_cycle: 51,
        end_cycle: 75,
        slope: 15,
        angle: 5,
    };
    send_projectile(player, &player.world(), &proj).await;

    send_message!(player, "Fired projectile (gfx {}) at NPC index {}.", gfx, npc_index);
}
