use proc_macro::TokenStream;

mod command;
mod data_provider;
mod dialogue_tree;
mod enum_data;
mod interaction;
mod message;
mod npc_action;
mod player_action;
mod player_system;
mod special_attack;

#[proc_macro_attribute]
pub fn message_decoder(attr: TokenStream, item: TokenStream) -> TokenStream {
    message::message_decoder(attr, item)
}

#[proc_macro_attribute]
pub fn message_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    message::message_handler(attr, item)
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    command::command(attr, item)
}

#[proc_macro_attribute]
pub fn npc_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    npc_action::npc_action(attr, item)
}

#[proc_macro_attribute]
pub fn special_attack(attr: TokenStream, item: TokenStream) -> TokenStream {
    special_attack::special_attack(attr, item)
}

#[proc_macro_attribute]
pub fn player_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    player_action::player_action(attr, item)
}

#[proc_macro_attribute]
pub fn player_system(attr: TokenStream, item: TokenStream) -> TokenStream {
    player_system::player_system(attr, item)
}

#[proc_macro_attribute]
pub fn data_provider(attr: TokenStream, item: TokenStream) -> TokenStream {
    data_provider::data_provider(attr, item)
}

#[proc_macro_attribute]
pub fn enum_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    enum_data::enum_data(attr, item)
}

#[proc_macro]
pub fn dialogue_tree(input: TokenStream) -> TokenStream {
    dialogue_tree::dialogue_tree(input)
}

#[proc_macro_attribute]
pub fn on_loc(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::on_loc(attr, item)
}

#[proc_macro_attribute]
pub fn on_npc(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::on_npc(attr, item)
}

#[proc_macro_attribute]
pub fn on_obj(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::on_obj(attr, item)
}

#[proc_macro_attribute]
pub fn on_player(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::on_player(attr, item)
}

#[proc_macro_attribute]
pub fn on_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::on_interface(attr, item)
}
