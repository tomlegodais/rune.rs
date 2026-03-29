use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, base_macros, emit_content_handler, extract_params};

pub fn on_player_click(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);
    let option = match attr.option_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let params = extract_params(&func);
    let player = params
        .first()
        .cloned()
        .unwrap_or_else(|| format_ident!("_player"));

    let target_p = params
        .get(1)
        .cloned()
        .unwrap_or_else(|| format_ident!("_player_index"));

    let base = base_macros();

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Player(#option) },
        quote! { let crate::player::InteractionTarget::Player { index: __player_index } = target else { unreachable!() }; },
        quote! {
            let mut #player = crate::player::PlayerRef;
            let #target_p = __player_index;
        },
        base,
        &func.block,
    )
}
