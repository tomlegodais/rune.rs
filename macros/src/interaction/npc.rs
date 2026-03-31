use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, emit_content_handler};

pub fn on_npc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);
    let npc_id = match attr.require_int("npc_id") {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let op = match attr.op_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let base = super::macros::base();
    let npc = super::macros::npc::macros();

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Npc(#npc_id, #op) },
        quote! { let crate::player::InteractionTarget::Npc { index: __npc_index } = target else { unreachable!() }; },
        quote! {
            let mut player = crate::player::PlayerRef;
            let npc_index = __npc_index;
        },
        quote! { #base #npc },
        &func.block,
    )
}
