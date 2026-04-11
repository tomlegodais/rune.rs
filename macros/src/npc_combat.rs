use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

use crate::interaction::InteractionAttr;

pub fn npc_combat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    if let Err(e) = attr.validate_keys(&["npc_id"]) {
        return e.to_compile_error().into();
    }

    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_body = &func.block;
    let wrapper_name = format_ident!("__{}_npc_combat", func_name);

    let npc_id = match attr.require_int("npc_id") {
        Ok(v) => v.clone(),
        Err(e) => return e.to_compile_error().into(),
    };

    quote! {
        fn #func_name(
            npc: &mut crate::npc::Npc,
            target: crate::content::CombatTarget,
            world: &crate::world::World,
        ) -> crate::content::NpcAttackResult {
            #func_body
        }

        #[allow(non_upper_case_globals)]
        static #wrapper_name: () = {
            inventory::submit! {
                crate::content::NpcCombatScript {
                    npc_id: #npc_id,
                    attack: #func_name,
                }
            }
        };
    }
    .into()
}
