mod client_cheat;
mod dialogue;
mod dispatch;
mod ifmoveslot;
mod ifsubclosed;
mod interaction;
mod message_public;
mod moveclick;
mod objstack;

use std::{any::TypeId, collections::HashMap, future::Future, pin::Pin};

pub use dispatch::{ContentHandler, ContentTarget, dispatch, run_action};
pub use interaction::try_dispatch_obj;
use net::IncomingMessage;
pub use objstack::pickup_obj_stack;
use tracing::debug;

use crate::player::Player;

type HandlerFn = for<'a> fn(&'a mut Player, IncomingMessage) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub struct MessageHandler {
    pub type_id_fn: fn() -> TypeId,
    pub handle: HandlerFn,
}

inventory::collect!(MessageHandler);

static HANDLERS: std::sync::LazyLock<HashMap<TypeId, HandlerFn>> = std::sync::LazyLock::new(|| {
    let mut map = HashMap::new();
    for entry in inventory::iter::<MessageHandler> {
        map.insert((entry.type_id_fn)(), entry.handle);
    }
    map
});

pub async fn handle_incoming_message(player: &mut Player, msg: IncomingMessage) {
    let type_id = (*msg).type_id();
    match HANDLERS.get(&type_id) {
        Some(handler) => handler(player, msg).await,
        None => debug!("Unhandled incoming message: {:?}", type_id),
    }
}
