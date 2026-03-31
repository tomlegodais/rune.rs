use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! requires {
            (stat = $stat:ident, level = $lvl:expr) => {
                if crate::player::active_player().stat().level(crate::player::Stat::$stat) < $lvl {
                    send_message!(
                        "You need a {} level of {} to do that.",
                        stringify!($stat),
                        $lvl
                    );
                    return;
                }
            };
            (stat = $stat:ident, level = $lvl:expr, $msg:expr) => {
                if crate::player::active_player().stat().level(crate::player::Stat::$stat) < $lvl {
                    send_message!($msg);
                    return;
                }
            };
            (inv, slots = $n:expr) => {
                if crate::player::active_player().inv().free_slots() < $n {
                    send_message!("Your inventory is too full.");
                    return;
                }
            };
        }
        macro_rules! give_xp {
            (stat = $stat:ident, amount = $xp:expr) => {
                crate::player::active_player().stat_mut().add_xp(crate::player::Stat::$stat, $xp).await;
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
