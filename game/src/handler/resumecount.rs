use macros::message_handler;
use net::ResumeCountDialog;

use super::MessageHandler;
use crate::player::Player;

#[message_handler]
async fn handle_resume_count(player: &mut Player, msg: ResumeCountDialog) {
    let Some(handler) = player.count_prompt_mut().take() else { return };
    handler(player, msg.value).await;
}
