// Test handler for working out the skill action macro DSL.
// Not representative of final content — uses placeholder IDs and hardcoded values.
#[macros::on_loc(id = 37312, op = Op1)]
async fn mine_gold_rock() {
    requires!(stat = Mining, level = 40);
    requires!(inv, slots = 1);

    send_message!("You swing your pickaxe at the rock.");

    repeat!(delay = 3, seq = 12189, {
        requires!(inv, slots = 1);

        if !successful!(chance = 0.20) {
            continue;
        }

        inv_add!(id = 444);
        give_xp!(stat = Mining, amount = 65.0);
        send_message!("You mine some gold ore.");

        if depleted!(chance = 0.25) {
            break;
        }
    });

    send_message!("The rock has been depleted.");
}
