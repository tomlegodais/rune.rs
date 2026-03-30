use std::{collections::HashMap, future::Future, pin::Pin};

use macros::message_handler;
use net::{ClickOption, NpcClick, ObjectClick, PlayerClick};

use super::MessageHandler;
use crate::{
    player::{InteractionTarget, Player},
    send_message, with_movement,
    world::Position,
};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ContentTarget {
    Object(u16, ClickOption),
    Npc(u16, ClickOption),
    Player(ClickOption),
    Item(i32, ClickOption),
}

pub type ContentHandlerFn = fn(InteractionTarget) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct ContentHandler {
    pub target: ContentTarget,
    pub handle: ContentHandlerFn,
}

inventory::collect!(ContentHandler);

static CONTENT_HANDLERS: std::sync::LazyLock<HashMap<ContentTarget, ContentHandlerFn>> =
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
        InteractionTarget::Item { .. } => return None,
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

pub fn dispatch_item(player: &mut Player, slot: u16, option: ClickOption) {
    if !try_dispatch_item(player, slot, option) {
        send_message!(player, "Nothing interesting happens.");
    }
}

pub fn try_dispatch_item(player: &mut Player, slot: u16, option: ClickOption) -> bool {
    let Some(item) = player.inventory().slot(slot as usize) else {
        return false;
    };

    let handler = CONTENT_HANDLERS
        .get(&ContentTarget::Item(item.id as i32, option))
        .or_else(|| CONTENT_HANDLERS.get(&ContentTarget::Item(-1, option)));

    let Some(handler) = handler else {
        return false;
    };

    let future = handler(crate::player::InteractionTarget::Item { slot });
    let shared = std::sync::Arc::new(crate::player::ActionShared::new());
    crate::player::set_action_context(player as *mut Player, shared.clone());

    let mut action_state = crate::player::ActionState { active: future, shared };

    let poll_result = crate::player::poll_action(&mut action_state);
    crate::player::clear_action_context();

    if poll_result.is_pending() {
        player.world().action_states.lock().insert(player.index, action_state);
    }

    true
}

#[message_handler]
async fn handle_object(player: &mut Player, msg: ObjectClick) {
    if crate::player::is_action_locked(player) {
        return;
    }
    player.world().action_states.lock().remove(&player.index);

    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    player.interaction_mut().set(
        InteractionTarget::Object {
            id: msg.id,
            x: dest.x,
            y: dest.y,
        },
        msg.option,
    );

    let params = crate::provider::get_collision().resolve_object_params(dest, msg.id as u32);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, dest, msg.ctrl_run, Some(params))
        .await);
}

#[message_handler]
async fn handle_npc(player: &mut Player, msg: NpcClick) {
    if crate::player::is_action_locked(player) {
        return;
    }
    player.world().action_states.lock().remove(&player.index);

    let index = msg.npc_index as usize;
    let world = player.world();
    if !world.npcs.contains(index) {
        return;
    }

    let (npc_pos, npc_id) = {
        let npc = world.npc(index);
        (npc.position, npc.npc_id)
    };

    let size = crate::provider::get_npc_definition(npc_id as u32)
        .map(|d| d.size as i32)
        .unwrap_or(1);

    world.npc_mut(index).entity.face_target = Some(player.index as u16 + 32768);
    player.entity.face_target = Some(index as u16);

    player
        .interaction_mut()
        .set(InteractionTarget::Npc { index }, msg.option);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, npc_pos, msg.ctrl_run, Some((size, size, 0)))
        .await);
}

#[message_handler]
async fn handle_player(player: &mut Player, msg: PlayerClick) {
    if crate::player::is_action_locked(player) {
        return;
    }
    player.world().action_states.lock().remove(&player.index);

    let index = msg.player_index as usize;
    let world = player.world();

    if !world.players.contains(index) || index == player.index {
        return;
    }

    let target_pos = world.player(index).position;

    player
        .interaction_mut()
        .set(InteractionTarget::Player { index }, msg.option);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, target_pos, msg.ctrl_run, Some((1, 1, 0)))
        .await);
}
