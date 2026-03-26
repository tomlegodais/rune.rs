use crate::player::Player;
use crate::send_message;
use macros::on_npc_click;

#[on_npc_click(npc_id = 2, option = 1)]
async fn talk_to_man(player: &mut Player, npc_index: usize) {
    let world = player.world();
    world.npc_mut(npc_index).force_talk("Hello!".to_string());
    send_message!(player, "The man waves at you.");
}
