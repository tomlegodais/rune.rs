use super::MessageHandler;
use crate::player::{Interaction, InteractionTarget, Player};
use crate::world::Position;
use crate::{send_message, with_movement};
use macros::message_handler;
use net::{ClickOption, NpcClick, ObjectClick, PlayerClick};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ContentTarget {
    Object(u16, ClickOption),
    Npc(u16, ClickOption),
    Player(ClickOption),
}

pub type ContentHandlerFn =
    fn(InteractionTarget) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

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
    };

    match CONTENT_HANDLERS.get(&content_target) {
        Some(handler) => Some(handler(target)),
        None => {
            send_message!(player, "Nothing interesting happens.");
            None
        }
    }
}

#[message_handler]
async fn handle_object(player: &mut Player, msg: ObjectClick) {
    if crate::player::is_action_locked(player) { return; }
    player.world().action_states.lock().remove(&player.index);

    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    player.systems.get_mut::<Interaction>().set(
        InteractionTarget::Object {
            id: msg.id,
            x: dest.x,
            y: dest.y,
        },
        msg.option,
    );

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, dest, msg.ctrl_run, true)
        .await);
}

#[message_handler]
async fn handle_npc(player: &mut Player, msg: NpcClick) {
    if crate::player::is_action_locked(player) { return; }
    player.world().action_states.lock().remove(&player.index);

    let index = msg.npc_index as usize;
    let world = player.world();
    let Some(npc_pos) = world
        .npcs
        .contains(index)
        .then(|| world.npc(index).position)
    else {
        return;
    };

    world.npc_mut(index).entity.face_target = Some(player.index as u16 + 32768);
    player.entity.face_target = Some(index as u16);

    player
        .systems
        .get_mut::<Interaction>()
        .set(InteractionTarget::Npc { index }, msg.option);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, npc_pos, msg.ctrl_run, true)
        .await);
}

#[message_handler]
async fn handle_player(player: &mut Player, msg: PlayerClick) {
    if crate::player::is_action_locked(player) { return; }
    player.world().action_states.lock().remove(&player.index);

    let index = msg.player_index as usize;
    let world = player.world();

    if !world.players.contains(index) || index == player.index {
        return;
    }

    let target_pos = world.player(index).position;

    player
        .systems
        .get_mut::<Interaction>()
        .set(InteractionTarget::Player { index }, msg.option);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, target_pos, msg.ctrl_run, true)
        .await);
}
