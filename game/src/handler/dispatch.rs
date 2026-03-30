use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use net::ClickOption;

use crate::{
    player::{
        ActionShared, ActionState, InteractionTarget, Player, clear_action_context, poll_action, set_action_context,
    },
    send_message,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ContentTarget {
    Object(u16, ClickOption),
    Npc(u16, ClickOption),
    Player(ClickOption),
    Item(i32, ClickOption),
    Button(Option<ClickOption>, u16, Option<u16>),
}

pub type ContentHandlerFn = fn(InteractionTarget) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct ContentHandler {
    pub target: ContentTarget,
    pub handle: ContentHandlerFn,
}

inventory::collect!(ContentHandler);

pub static CONTENT_HANDLERS: std::sync::LazyLock<HashMap<ContentTarget, ContentHandlerFn>> =
    std::sync::LazyLock::new(|| {
        inventory::iter::<ContentHandler>()
            .map(|entry| (entry.target, entry.handle))
            .collect()
    });

pub fn dispatch(
    player: &mut Player,
    target: InteractionTarget,
    option: ClickOption,
) -> Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> {
    let content_target = match &target {
        InteractionTarget::Object { id, .. } => ContentTarget::Object(*id, option),
        InteractionTarget::Npc { index } => {
            let world = player.world();
            if !world.npcs.contains(*index) {
                return None;
            }
            let npc_id = world.npc(*index).npc_id;
            ContentTarget::Npc(npc_id, option)
        }
        InteractionTarget::Player { .. } => ContentTarget::Player(option),
        InteractionTarget::Item { .. } | InteractionTarget::Button { .. } => return None,
        InteractionTarget::GroundItem { .. } => {
            return Some(Box::pin(crate::handler::pickup_ground_item(target)));
        }
    };

    match CONTENT_HANDLERS.get(&content_target) {
        Some(handler) => Some(handler(target)),
        None => {
            send_message!(player, "Nothing interesting happens.");
            None
        }
    }
}

pub fn run_action(player: &mut Player, future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
    let shared = Arc::new(ActionShared::new());
    set_action_context(player as *mut Player, shared.clone());

    let mut action_state = ActionState { active: future, shared };
    let poll_result = poll_action(&mut action_state);
    clear_action_context();

    if poll_result.is_pending() {
        player.world().action_states.lock().insert(player.index, action_state);
    }
}
