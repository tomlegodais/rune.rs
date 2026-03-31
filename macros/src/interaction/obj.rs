use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, emit_content_handler};

pub fn on_obj(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);

    let id_expr = match attr.get_int("id") {
        Some(id) => quote! { #id as i32 },
        None => quote! { -1i32 },
    };

    let op = match attr.op_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let base = super::macros::base();
    let obj = super::macros::obj::macros();

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Obj(#id_expr, #op) },
        quote! { let crate::player::InteractionTarget::Obj { slot: __slot } = target else { unreachable!() }; },
        quote! {
            let mut player = crate::player::PlayerRef;
            let slot = __slot;
        },
        quote! { #base #obj },
        &func.block,
    )
}
