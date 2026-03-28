use proc_macro::TokenStream;

mod command;
mod data_provider;
mod interaction;
mod message;
mod player_system;

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
pub fn player_system(attr: TokenStream, item: TokenStream) -> TokenStream {
    player_system::player_system(attr, item)
}

#[proc_macro_attribute]
pub fn data_provider(attr: TokenStream, item: TokenStream) -> TokenStream {
    data_provider::data_provider(attr, item)
}

#[proc_macro_attribute]
pub fn on_object_click(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::object::on_object_click(attr, item)
}

#[proc_macro_attribute]
pub fn on_npc_click(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::npc::on_npc_click(attr, item)
}

#[proc_macro_attribute]
pub fn on_item_option(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::item::on_item_option(attr, item)
}

#[proc_macro_attribute]
pub fn on_player_click(attr: TokenStream, item: TokenStream) -> TokenStream {
    interaction::player::on_player_click(attr, item)
}
