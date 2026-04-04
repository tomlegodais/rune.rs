use macros::command;

use super::CommandEntry;
use crate::{
    entity::{Hit, HitType},
    player::{Clientbound, Player},
    send_message,
};

#[command(name = "ndamage")]
async fn ndamage(player: &mut Player, index: usize, amount: u16) {
    let world = player.world();
    if !world.npcs.contains(index) {
        player.send_message("NPC not found").await;
        return;
    }
    let died = world.npc_mut(index).damage(Hit::new(amount, HitType::Normal));
    if died {
        send_message!(player, "NPC {} died.", index);
    } else {
        let npc = world.npc(index);
        send_message!(
            player,
            "Dealt {} to NPC {} ({}/{}hp).",
            amount,
            index,
            npc.current_hp,
            npc.max_hp
        );
    }
}

#[command(name = "damage")]
async fn damage(player: &mut Player, amount: u8) {
    let hit_type = if amount == 0 { HitType::Block } else { HitType::Normal };
    let died = player.hitpoints_mut().damage(Hit::new(amount as u16, hit_type));
    if died {
        send_message!(player, "You took {} damage and died.", amount);
    } else {
        send_message!(
            player,
            "You took {} damage. HP: {}/{}",
            amount,
            player.hitpoints().current(),
            player.hitpoints().max()
        );
    }
}
