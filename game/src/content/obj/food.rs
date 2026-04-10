const EAT_SEQ: u16 = 829;
const EAT_DELAY: u16 = 3;

async fn eat(player: &mut crate::player::Player, slot: u16, heal: u8, name: &str) {
    use crate::player::Clientbound;

    player.inv_mut().clear_slot(slot as usize).await;
    player.hitpoints_mut().heal(heal);
    player.seq(EAT_SEQ);
    player.combat_mut().set_eat_delay(EAT_DELAY);
    player.send_message(format!("You eat the {}.", name)).await;
}

macro_rules! food {
    ($fn_name:ident, $obj_id:expr, $heal:expr, $name:expr) => {
        #[macros::on_obj(id = $obj_id, op = Op1)]
        async fn $fn_name() {
            eat(&mut player, slot, $heal, $name).await;
        }
    };
}

food!(eat_shrimps, 315, 3, "shrimps");
food!(eat_anchovies, 319, 3, "anchovies");
food!(eat_sardine, 325, 4, "sardine");
food!(eat_herring, 329, 5, "herring");
food!(eat_trout, 333, 5, "trout");
food!(eat_salmon, 339, 7, "salmon");
food!(eat_bass, 347, 8, "bass");
food!(eat_pike, 351, 7, "pike");
food!(eat_cod, 339, 7, "cod");
food!(eat_tuna, 361, 10, "tuna");
food!(eat_lobster, 379, 12, "lobster");
food!(eat_swordfish, 373, 14, "swordfish");
food!(eat_monkfish, 7946, 16, "monkfish");
food!(eat_shark, 385, 20, "shark");
food!(eat_manta_ray, 391, 22, "manta ray");
food!(eat_sea_turtle, 397, 21, "sea turtle");
food!(eat_dark_crab, 11936, 22, "dark crab");
food!(eat_bread, 2309, 5, "bread");
food!(eat_cake, 1891, 4, "cake");
food!(eat_2_3_cake, 1893, 4, "cake");
food!(eat_slice_cake, 1895, 4, "cake");
food!(eat_cooked_chicken, 2140, 3, "chicken");
food!(eat_cooked_meat, 2142, 3, "meat");
food!(eat_potato_cheese, 6705, 16, "potato with cheese");
food!(eat_karambwan, 3144, 18, "karambwan");
