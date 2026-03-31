use macros::command;

use crate::{command::CommandEntry, player::Player, send_message};

#[command(name = "seq")]
async fn seq(player: &mut Player, id: u16, speed: Option<u8>) {
    player.seq(id).speed(speed.unwrap_or(0));
}

#[command(name = "spotanim")]
async fn spotanim(player: &mut Player, id: u16, speed: Option<u16>, height: Option<u16>, rotation: Option<u8>) {
    player
        .spot_anim(id)
        .speed(speed.unwrap_or(0))
        .height(height.unwrap_or(0))
        .rotation(rotation.unwrap_or(0));
}

#[command(name = "npc_seq")]
async fn npc_seq(player: &mut Player, npc_index: usize, id: u16, speed: Option<u8>) {
    let world = player.world();
    if world.npcs.contains(npc_index) {
        world.npc_mut(npc_index).seq(id).speed(speed.unwrap_or(0));
    } else {
        send_message!(player, "No NPC at index {}", npc_index);
    }
}

#[command(name = "npc_spotanim")]
async fn npc_spotanim(
    player: &mut Player,
    npc_index: usize,
    id: u16,
    speed: Option<u16>,
    height: Option<u16>,
    rotation: Option<u8>,
) {
    let world = player.world();
    if world.npcs.contains(npc_index) {
        world
            .npc_mut(npc_index)
            .spot_anim(id)
            .speed(speed.unwrap_or(0))
            .height(height.unwrap_or(0))
            .rotation(rotation.unwrap_or(0));
    } else {
        send_message!(player, "No NPC at index {}", npc_index);
    }
}
