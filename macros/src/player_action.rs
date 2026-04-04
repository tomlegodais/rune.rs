use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn player_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_body = &func.block;
    let macros = crate::interaction::macros::base();

    quote! {
        #[allow(unused_macros, unused_variables)]
        fn #func_name() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
            use crate::player::Clientbound as _;
            Box::pin(async move {
                let __shared = crate::player::active_shared();
                let mut player = crate::player::PlayerRef;
                #macros
                #func_body
            })
        }
    }
    .into()
}
