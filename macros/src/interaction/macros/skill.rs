use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! requires {
            (skill = $skill:ident, level = $lvl:expr) => {
                if crate::player::active_player().skill().level(crate::player::Skill::$skill) < $lvl {
                    send_message!(
                        "You need a {} level of {} to do that.",
                        stringify!($skill),
                        $lvl
                    );
                    return;
                }
            };
            (skill = $skill:ident, level = $lvl:expr, $msg:expr) => {
                if crate::player::active_player().skill().level(crate::player::Skill::$skill) < $lvl {
                    send_message!($msg);
                    return;
                }
            };
            (inventory, slots = $n:expr) => {
                if crate::player::active_player().inv().free_slots() < $n {
                    send_message!("Your inventory is too full.");
                    return;
                }
            };
        }
        macro_rules! give_xp {
            (skill = $skill:ident, amount = $xp:expr) => {
                crate::player::active_player().skill_mut().add_xp(crate::player::Skill::$skill, $xp).await;
            };
        }
        macro_rules! successful {
            (chance = $chance:expr) => {
                rand::random::<f64>() < $chance
            };
        }
        macro_rules! depleted {
            (chance = $chance:expr) => {
                rand::random::<f64>() < $chance
            };
        }
    }
}
