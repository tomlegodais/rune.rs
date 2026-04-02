use std::{future::Future, pin::Pin};

use macros::message_handler;
use net::{Op, OpObj};

use super::MessageHandler;
use crate::player::{Clientbound, InteractionTarget, Player};

#[message_handler]
async fn handle_op_obj(player: &mut Player, msg: OpObj) {
    if crate::player::is_action_locked(player) {
        return;
    }

    player.cancel_action().await;

    let (id, position) = {
        let world = player.world();
        let Some(id) = world
            .obj_stacks
            .find(msg.obj_id, msg.x as i32, msg.y as i32, player.index)
        else {
            return;
        };

        let Some(snap) = world.obj_stacks.get(id) else {
            return;
        };
        (id, snap.position)
    };

    player
        .interaction_mut()
        .set(InteractionTarget::ObjStack { id, position }, Op::Op1);

    player.movement_mut().walk_to(position, msg.ctrl_run, None).await;
}

pub fn pickup_obj_stack(target: InteractionTarget) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    let InteractionTarget::ObjStack { id, position } = target else { unreachable!() };

    Box::pin(async move {
        let player = crate::player::active_player();
        let (obj_id, amount, owner, private_ticks_remaining, public_ticks_remaining, other_indices) = {
            let world = player.world();
            let Some(snap) = world.obj_stacks.get(id) else {
                return;
            };

            let other_indices: Vec<usize> = world
                .players
                .keys()
                .into_iter()
                .filter(|&i| i != player.index)
                .collect();

            world.obj_stacks.remove(id);
            (
                snap.obj_id,
                snap.amount,
                snap.owner,
                snap.private_ticks_remaining,
                snap.public_ticks_remaining,
                other_indices,
            )
        };

        player.obj_stack_mut().forget(id, obj_id, position).await;

        for index in other_indices {
            let world = player.world();
            let mut p = world.players.get_mut(index);
            p.obj_stack_mut().forget(id, obj_id, position).await;
        }

        let remainder = player.inv_mut().add(obj_id, amount).await;

        if remainder > 0 {
            player.world().obj_stacks.add_with_state(
                obj_id,
                remainder,
                position,
                owner,
                private_ticks_remaining,
                public_ticks_remaining,
            );

            player.send_message("You can't carry any more of that.").await;
        }
    })
}
