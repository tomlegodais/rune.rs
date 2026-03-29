use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, base_macros, emit_content_handler, extract_params};

pub fn on_item_option(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);

    let id_expr = match attr.get("id") {
        Some(id) => quote! { #id as i32 },
        None => quote! { -1i32 },
    };

    let option = match attr.option_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let params = extract_params(&func);
    let player = params.first().cloned().unwrap_or_else(|| format_ident!("_player"));

    let slot_p = params.get(1).cloned().unwrap_or_else(|| format_ident!("_slot"));

    let base = base_macros();
    let item_m = quote! {
        macro_rules! remove_item {
            ($amount:expr) => {
                crate::player::active_player()
                    .inventory_mut()
                    .remove_item(__slot as usize, $amount)
                    .await
            };
        }
        macro_rules! slot_item {
            () => { crate::player::active_player().inventory().slot(__slot as usize) };
        }
        macro_rules! clear_slot {
            () => { crate::player::active_player().inventory_mut().clear_slot(__slot as usize).await };
        }
        macro_rules! drop_to_ground {
            ($item_id:expr, $amount:expr) => {{
                let __player = crate::player::active_player();
                let __pos = __player.position;
                let __world = __player.world();
                __player.ground_item_mut().drop($item_id, $amount, __pos, &__world);
            }};
        }
    };

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Item(#id_expr, #option) },
        quote! { let crate::player::InteractionTarget::Item { slot: __slot } = target else { unreachable!() }; },
        quote! {
            let mut #player = crate::player::PlayerRef;
            let #slot_p = __slot;
        },
        quote! { #base #item_m },
        &func.block,
    )
}
