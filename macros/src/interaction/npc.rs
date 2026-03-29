use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

use super::{InteractionAttr, base_macros, emit_content_handler, extract_params};

pub fn on_npc_click(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    let func = parse_macro_input!(item as syn::ItemFn);
    let wrapper_name = format_ident!("__{}_content_wrapper", func.sig.ident);
    let npc_id = match attr.require("npc_id") {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let option = match attr.option_variant() {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let params = extract_params(&func);
    let player = params.first().cloned().unwrap_or_else(|| format_ident!("_player"));

    let npc = params.get(1).cloned().unwrap_or_else(|| format_ident!("_npc_index"));

    let base = base_macros();
    let npc_m = quote! {
        macro_rules! npc_force_talk { ($($a:tt)*) => { crate::player::npc_force_talk(&*#player, #npc, &format!($($a)*)) }; }
        macro_rules! npc_anim {
            ($id:expr) => { drop(#player.world().npc_mut(#npc).anim($id)) };
            ($id:expr, $($k:ident = $v:expr),+) => { drop({ let b = #player.world().npc_mut(#npc).anim($id); $(let b = b.$k($v);)+ b }) };
        }
        macro_rules! npc_spotanim {
            ($id:expr) => { drop(#player.world().npc_mut(#npc).spot_anim($id)) };
            ($id:expr, $($k:ident = $v:expr),+) => { drop({ let b = #player.world().npc_mut(#npc).spot_anim($id); $(let b = b.$k($v);)+ b }) };
        }
    };

    emit_content_handler(
        &wrapper_name,
        quote! { crate::handler::ContentTarget::Npc(#npc_id, #option) },
        quote! { let crate::player::InteractionTarget::Npc { index: __npc_index } = target else { unreachable!() }; },
        quote! {
            let mut #player = crate::player::PlayerRef;
            let #npc = __npc_index;
        },
        quote! { #base #npc_m },
        &func.block,
    )
}
