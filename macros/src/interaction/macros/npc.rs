use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! npc_force_talk { ($($a:tt)*) => { crate::player::npc_force_talk(&*player, npc_index, &format!($($a)*)) }; }
        macro_rules! npc_anim {
            ($id:expr) => { drop(player.world().npc_mut(npc_index).anim($id)) };
            ($id:expr, $($k:ident = $v:expr),+) => { drop({ let b = player.world().npc_mut(npc_index).anim($id); $(let b = b.$k($v);)+ b }) };
        }
        macro_rules! npc_spotanim {
            ($id:expr) => { drop(player.world().npc_mut(npc_index).spot_anim($id)) };
            ($id:expr, $($k:ident = $v:expr),+) => { drop({ let b = player.world().npc_mut(npc_index).spot_anim($id); $(let b = b.$k($v);)+ b }) };
        }
    }
}
