use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use net::Op;

use crate::{
    player::{
        ActionShared, ActionState, InteractionTarget, Player, clear_action_context, poll_action, set_action_context,
    },
    send_message,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ContentTarget {
    Loc(u16, Op),
    Npc(u16, Op),
    Player(Op),
    Obj(i32, Op),
    Button(Option<Op>, u16, Option<u16>),
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
    op: Op,
) -> Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> {
    let content_target = match &target {
        InteractionTarget::Loc { id, .. } => ContentTarget::Loc(*id, op),
        InteractionTarget::Npc { index } => {
            let world = player.world();
            if !world.npcs.contains(*index) {
                return None;
            }
            let npc_id = world.npc(*index).npc_id;
            ContentTarget::Npc(npc_id, op)
        }
        InteractionTarget::Player { .. } => ContentTarget::Player(op),
        InteractionTarget::Obj { .. } | InteractionTarget::Button { .. } => return None,
        InteractionTarget::ObjStack { .. } => {
            return Some(Box::pin(crate::handler::pickup_obj_stack(target)));
        }
    };

    match CONTENT_HANDLERS.get(&content_target) {
        Some(handler) => Some(handler(target)),
        None => {
            if let InteractionTarget::Loc { id, x, y } = &target {
                tracing::debug!(loc_id = id, x, y, op = ?op, "unhandled loc interaction");
            }
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
