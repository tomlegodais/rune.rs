use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! inv_add {
            (id = $id:expr) => {
                crate::player::active_player().inv_mut().add($id, 1).await;
            };
            (id = $id:expr, amount = $n:expr) => {
                crate::player::active_player().inv_mut().add($id, $n).await;
            };
        }
        macro_rules! inv_full {
            () => {
                crate::player::active_player().inv().free_slots() == 0
            };
        }
    }
}
