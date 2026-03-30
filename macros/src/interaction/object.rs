use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, base_macros, emit_content_handler};

pub fn on_object(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);
    let id = match attr.require_int("id") {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let option = match attr.option_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let base = base_macros();

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Object(#id, #option) },
        quote! { let crate::player::InteractionTarget::Object { id: __id, x: __x, y: __y } = target else { unreachable!() }; },
        quote! {
            let mut player = crate::player::PlayerRef;
            let obj_id = __id;
            let obj_x = __x;
            let obj_y = __y;
        },
        base,
        &func.block,
    )
}
