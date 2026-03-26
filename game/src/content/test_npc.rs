#[macros::on_npc_click(npc_id = 2, option = 1)]
async fn talk_to_man(player: &mut crate::player::Player, npc_index: usize) {
    npc_force_talk!("Hello!");
    send_message!("The man waves at you.");
    delay!(2).await;
    npc_force_talk!("How are you?");
    send_message!("He seems friendly.");
}
