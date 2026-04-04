use macros::message_handler;
use net::{ExamLoc, IfButton, Op, OpLoc, OpNpc, OpPlayer};

use super::{
    MessageHandler,
    dispatch::{CONTENT_HANDLERS, ContentTarget, run_action},
};
use crate::{
    entity::WalkTarget,
    player::{Clientbound, InteractionTarget, Player, is_action_locked},
    world::Position,
};

#[message_handler]
async fn handle_oploc(player: &mut Player, msg: OpLoc) {
    if is_action_locked(player) {
        return;
    }

    player.cancel_action(true).await;

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
    player
        .movement_mut()
        .walk_to(dest, msg.ctrl_run, Some(WalkTarget::Loc(params)))
        .await;
}

#[message_handler]
async fn handle_opnpc(player: &mut Player, msg: OpNpc) {
    if is_action_locked(player) {
        return;
    }

    player.cancel_action(true).await;

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
    player.interaction_mut().set(InteractionTarget::Npc { index }, msg.op);
    player
        .movement_mut()
        .walk_to(
            npc_pos,
            msg.ctrl_run,
            Some(WalkTarget::Rect {
                width: size,
                height: size,
                access: 0,
            }),
        )
        .await;
}

#[message_handler]
async fn handle_opplayer(player: &mut Player, msg: OpPlayer) {
    if is_action_locked(player) {
        return;
    }

    player.cancel_action(true).await;

    let index = msg.player_index as usize;
    let world = player.world();
    if !world.players.contains(index) || index == player.index {
        return;
    }

    let target_pos = world.player(index).position;
    player
        .interaction_mut()
        .set(InteractionTarget::Player { index }, msg.op);

    player
        .movement_mut()
        .walk_to(
            target_pos,
            msg.ctrl_run,
            Some(WalkTarget::Rect {
                width: 1,
                height: 1,
                access: 0,
            }),
        )
        .await;
}

#[message_handler]
async fn handle_examloc(player: &mut Player, msg: ExamLoc) {
    let name = crate::provider::get_loc_type(msg.id as u32)
        .map(|d| d.name.as_str())
        .unwrap_or("null");

    player.send_message(format!("It's a {}.", name)).await;
}

#[message_handler]
async fn handle_ifbutton(player: &mut Player, msg: IfButton) {
    let handler = [Some(msg.op), None]
        .into_iter()
        .flat_map(|o| [Some(msg.component), None].map(|c| (o, c)))
        .find_map(|(o, c)| CONTENT_HANDLERS.get(&ContentTarget::Button(o, msg.interface, c)));

    let Some(handler) = handler else {
        tracing::debug!(
            op = ?msg.op,
            interface = msg.interface,
            component = msg.component,
            slot1 = msg.slot1,
            slot2 = msg.slot2,
            "Unhandled IfButton"
        );
        return;
    };

    if is_action_locked(player) {
        return;
    }

    player.cancel_action(false).await;

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
