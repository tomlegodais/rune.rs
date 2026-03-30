use macros::message_handler;
use net::{ButtonClick, ClickOption, NpcClick, ObjectClick, PlayerClick};

use super::{
    MessageHandler,
    dispatch::{CONTENT_HANDLERS, ContentTarget, run_action},
};
use crate::{
    player::{InteractionTarget, Player, is_action_locked},
    with_movement,
    world::Position,
};

#[message_handler]
async fn handle_object(player: &mut Player, msg: ObjectClick) {
    if is_action_locked(player) {
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
    if is_action_locked(player) {
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
    if is_action_locked(player) {
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

#[message_handler]
async fn handle_button(player: &mut Player, msg: ButtonClick) {
    let handler = [Some(msg.option), None]
        .into_iter()
        .flat_map(|o| [Some(msg.component), None].map(|c| (o, c)))
        .find_map(|(o, c)| CONTENT_HANDLERS.get(&ContentTarget::Button(o, msg.interface, c)));

    let Some(handler) = handler else {
        return;
    };

    if is_action_locked(player) {
        return;
    }

    player.world().action_states.lock().remove(&player.index);

    let target = InteractionTarget::Button {
        interface: msg.interface,
        component: msg.component,
        option: msg.option,
        slot1: msg.slot1,
        slot2: msg.slot2,
    };

    run_action(player, handler(target));
}

pub fn try_dispatch_item(player: &mut Player, option: ClickOption, slot: u16) -> bool {
    let Some(item) = player.inventory().slot(slot as usize) else {
        return false;
    };

    let handler = CONTENT_HANDLERS
        .get(&ContentTarget::Item(item.id as i32, option))
        .or_else(|| CONTENT_HANDLERS.get(&ContentTarget::Item(-1, option)));

    let Some(handler) = handler else {
        return false;
    };

    run_action(player, handler(InteractionTarget::Item { slot }));
    true
}
