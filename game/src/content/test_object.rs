#[macros::on_object(id = 37312, option = One)]
async fn mine_rock() {
    send_message!("You swing your pickaxe at the rock.");
    skill_action!()
        .interval(4)
        .anim(12189)
        .on_success(
            || rand::random::<u8>() < 48,
            || send_message!("You manage to mine some gold ore from the rock."),
        )
        .await;
    send_message!("The rock has been depleted.");
}
