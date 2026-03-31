use quote::quote;

mod seq;
mod control_flow;
mod inventory;
pub mod item;
mod messaging;
pub mod npc;
mod skill;

pub fn base() -> proc_macro2::TokenStream {
    let msg = messaging::macros();
    let inv = inventory::macros();
    let seq = seq::macros();
    let skill = skill::macros();
    let ctrl = control_flow::macros();
    quote! { #msg #inv #seq #skill #ctrl }
}
