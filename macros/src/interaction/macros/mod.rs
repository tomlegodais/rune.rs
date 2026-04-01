use quote::quote;

mod control_flow;
mod dialogue;
mod inv;
mod loc;
mod messaging;
pub mod npc;
pub mod obj;
mod seq;
mod stat;

pub fn base() -> proc_macro2::TokenStream {
    let msg = messaging::macros();
    let inv = inv::macros();
    let seq = seq::macros();
    let stat = stat::macros();
    let ctrl = control_flow::macros();
    let loc = loc::macros();
    let dialogue = dialogue::macros();
    quote! { #msg #inv #seq #stat #ctrl #loc #dialogue }
}
