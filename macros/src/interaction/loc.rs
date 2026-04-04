use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, emit_content_handler};

pub fn on_loc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    if let Err(e) = attr.validate_keys(&["op", "id"]) {
        return e.to_compile_error().into();
    }
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);
    let id = match attr.require_int("id") {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let op = match attr.op_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let base = super::macros::base();

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Loc(#id, #op) },
        quote! { let crate::player::InteractionTarget::Loc { id: __id, x: __x, y: __y } = target else { unreachable!() }; },
        quote! {
            let mut player = crate::player::PlayerRef;
            let loc_id = __id;
            let loc_x = __x;
            let loc_y = __y;
        },
        base,
        &func.block,
    )
}
