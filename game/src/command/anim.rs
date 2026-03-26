use crate::command::CommandEntry;
use crate::player::Player;
use crate::send_message;
use macros::command;

#[command(name = "anim")]
async fn anim(player: &mut Player, id: u16, speed: Option<u8>) {
    player.anim(id).speed(speed.unwrap_or(0));
}

#[command(name = "spotanim")]
async fn spotanim(
    player: &mut Player,
    id: u16,
    speed: Option<u16>,
    height: Option<u16>,
    rotation: Option<u8>,
) {
    player
        .spot_anim(id)
        .speed(speed.unwrap_or(0))
        .height(height.unwrap_or(0))
        .rotation(rotation.unwrap_or(0));
}

#[command(name = "npc_anim")]
async fn npc_anim(player: &mut Player, npc_index: usize, id: u16, speed: Option<u8>) {
    let world = player.world();
    if world.npcs.contains(npc_index) {
        world.npc_mut(npc_index).anim(id).speed(speed.unwrap_or(0));
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
