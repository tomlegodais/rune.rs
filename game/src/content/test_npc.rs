#[macros::on_npc_click(npc_id = 2, option = 1)]
async fn talk_to_man() {
    anim!(863);
    delay!(2);

    npc_anim!(863);
    npc_force_talk!("Hello!");
    send_message!("The man waves at you.");

    delay!(2);
    npc_force_talk!("How are you?");
    send_message!("He seems friendly.");
}