use quote::quote;

mod control_flow;
mod inv;
mod messaging;
pub mod npc;
pub mod obj;
mod seq;
mod skill;

pub fn base() -> proc_macro2::TokenStream {
    let msg = messaging::macros();
    let inv = inv::macros();
    let seq = seq::macros();
    let skill = skill::macros();
    let ctrl = control_flow::macros();
    quote! { #msg #inv #seq #skill #ctrl }
}
