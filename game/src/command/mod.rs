mod pos;
mod setlevel;
mod tele;

use crate::player::Player;
use crate::send_message;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

type CommandFn =
    for<'a> fn(&'a mut Player, &'a str) -> Pin<Box<dyn Future<Output=()> + Send + 'a>>;

pub struct CommandEntry {
    pub name: &'static str,
    pub handle: CommandFn,
}

inventory::collect!(CommandEntry);

static COMMANDS: std::sync::LazyLock<HashMap<&'static str, CommandFn>> =
    std::sync::LazyLock::new(|| {
        let mut map = HashMap::new();
        for entry in inventory::iter::<CommandEntry> {
            map.insert(entry.name, entry.handle);
        }
        map
    });

pub async fn dispatch(player: &mut Player, name: &str, args: &str) {
    match COMMANDS.get(name) {
        Some(handler) => handler(player, args).await,
        None => send_message!(player, "Unknown command: {}", name),
    }
}
