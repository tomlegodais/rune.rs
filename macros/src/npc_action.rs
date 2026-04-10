use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn npc_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_body = &func.block;
    let vis = &func.vis;
    let params = &func.sig.inputs;

    quote! {
        #[allow(unused_macros, unused_variables)]
        #vis fn #func_name(#params) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
            Box::pin(async move {
                let __shared = crate::npc::action::active_shared();
                let mut npc = crate::npc::action::NpcRef;
                macro_rules! delay { ($t:expr) => { crate::npc::action::delay(&__shared, $t).await }; }
                #func_body
            })
        }
    }
    .into()
}
