use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! anim {
            ($id:expr) => { crate::player::active_player().anim($id) };
            ($id:expr, $($k:ident = $v:expr),+) => { { let b = crate::player::active_player().anim($id); $(let b = b.$k($v);)+ b } };
        }
        macro_rules! spotanim {
            ($id:expr) => { crate::player::active_player().spot_anim($id) };
            ($id:expr, $($k:ident = $v:expr),+) => { { let b = crate::player::active_player().spot_anim($id); $(let b = b.$k($v);)+ b } };
        }
    }
}
