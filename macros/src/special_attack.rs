use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

use crate::interaction::InteractionAttr;

pub fn special_attack(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as InteractionAttr);
    if let Err(e) = attr.validate_keys(&["obj_id", "energy"]) {
        return e.to_compile_error().into();
    }

    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_body = &func.block;
    let wrapper_name = format_ident!("__{}_spec_wrapper", func_name);

    let obj_id = match attr.require_int("obj_id") {
        Ok(v) => v.clone(),
        Err(e) => return e.to_compile_error().into(),
    };

    let energy = match attr.require_int("energy") {
        Ok(v) => v.clone(),
        Err(e) => return e.to_compile_error().into(),
    };

    quote! {
        fn #func_name(
            player: &mut crate::player::Player,
            atk: &crate::content::combat::formula::MeleeAttack,
            def: &crate::content::combat::formula::MeleeDefence,
            atk_type: filesystem::AttackType,
            target: crate::content::combat::CombatTarget,
        ) -> crate::content::combat::special::SpecialResult {
            #func_body
        }

        #[allow(non_upper_case_globals)]
        static #wrapper_name: () = {
            inventory::submit! {
                crate::content::combat::special::SpecialAttackEntry {
                    obj_id: #obj_id,
                    energy_cost: #energy,
                    execute: #func_name,
                }
            }
        };
    }
    .into()
}
