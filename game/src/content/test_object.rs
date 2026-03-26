#[macros::on_object_click(id = 37312, option = 1)]
async fn mine_rock() {
    send_message!("You swing your pickaxe at the rock.");
    skill_action!()
        .interval(4)
        .on_attempt(|p| crate::player::send_message(p, "Swinging..."))
        .on_success(
            |_| rand::random::<u8>() < 48,
            |p| crate::player::send_message(p, "You manage to mine some gold ore from the rock."),
        )
        .await;
    send_message!("The rock has been depleted.");
}
