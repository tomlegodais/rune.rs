mod chat;
mod command;

use crate::player::Player;
use net::IncomingMessage;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tracing::debug;

type HandlerFn =
    for<'a> fn(&'a mut Player, IncomingMessage) -> Pin<Box<dyn Future<Output=()> + Send + 'a>>;

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

pub async fn handle(player: &mut Player, msg: IncomingMessage) {
    let type_id = (*msg).type_id();
    match HANDLERS.get(&type_id) {
        Some(handler) => handler(player, msg).await,
        None => debug!("[{}] No handler for message", player.username),
    }
}
