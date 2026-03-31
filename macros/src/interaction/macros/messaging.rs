use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! send_message { ($($a:tt)*) => { crate::player::send_message(crate::player::active_player(), &format!($($a)*)) }; }
    }
}
