use macros::message_handler;
use net::{ButtonClick, Op, LocClick, NpcClick, PlayerClick};

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
async fn handle_loc(player: &mut Player, msg: LocClick) {
    if is_action_locked(player) {
        return;
    }

    player.world().action_states.lock().remove(&player.index);

    let dest = Position::new(msg.x as i32, msg.y as i32, player.position.plane);
    player.interaction_mut().set(
        InteractionTarget::Loc {
            id: msg.id,
            x: dest.x,
            y: dest.y,
        },
        msg.op,
    );

    let params = crate::provider::get_collision().resolve_loc_params(dest, msg.id as u32);

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

    let size = crate::provider::get_npc_type(npc_id as u32)
        .map(|d| d.size as i32)
        .unwrap_or(1);

    world.npc_mut(index).entity.face_target = Some(player.index as u16 + 32768);
    player.entity.face_target = Some(index as u16);

    player
        .interaction_mut()
        .set(InteractionTarget::Npc { index }, msg.op);

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
        .set(InteractionTarget::Player { index }, msg.op);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, target_pos, msg.ctrl_run, Some((1, 1, 0)))
        .await);
}

#[message_handler]
async fn handle_button(player: &mut Player, msg: ButtonClick) {
    let handler = [Some(msg.op), None]
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
        op: msg.op,
        slot1: msg.slot1,
        slot2: msg.slot2,
    };

    run_action(player, handler(target));
}

pub fn try_dispatch_obj(player: &mut Player, op: Op, slot: u16) -> bool {
    let Some(obj) = player.inv().slot(slot as usize) else {
        return false;
    };

    let handler = CONTENT_HANDLERS
        .get(&ContentTarget::Obj(obj.id as i32, op))
        .or_else(|| CONTENT_HANDLERS.get(&ContentTarget::Obj(-1, op)));

    let Some(handler) = handler else {
        return false;
    };

    run_action(player, handler(InteractionTarget::Obj { slot }));
    true
}
