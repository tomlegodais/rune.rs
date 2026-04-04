use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, emit_content_handler};

pub fn on_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    if let Err(e) = attr.validate_keys(&["op", "interface", "component"]) {
        return e.to_compile_error().into();
    }
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);
    let op_expr = match attr.op_variant() {
        Ok(v) => quote! { Some(#v) },
        Err(_) => quote! { None },
    };

    let interface = match attr.require_int("interface") {
        Ok(v) => v.clone(),
        Err(e) => return e.to_compile_error().into(),
    };

    let component_key = attr.get_int("component");
    let target_expr = match &component_key {
        Some(c) => quote! { crate::handler::ContentTarget::Button(#op_expr, #interface, Some(#c)) },
        None => quote! { crate::handler::ContentTarget::Button(#op_expr, #interface, None) },
    };

    let base = super::macros::base();

    emit_content_handler(
        &wrapper_name,
        target_expr,
        quote! {
            let crate::player::InteractionTarget::Button {
                interface: __interface,
                component: __component,
                op: __op,
                slot1: __slot1,
                slot2: __slot2,
            } = target else { unreachable!() };
        },
        quote! {
            let mut player = crate::player::PlayerRef;
            let op = __op;
            let component = __component;
            let slot1 = __slot1;
            let slot2 = __slot2;
        },
        base,
        &func.block,
    )
}
