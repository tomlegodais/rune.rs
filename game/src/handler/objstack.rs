use std::{future::Future, pin::Pin};

use macros::message_handler;
use net::{ChatMessage, ClickOption, ObjClick, OutboxExt};

use super::MessageHandler;
use crate::{
    player::{InteractionTarget, Player},
    with_movement,
};

#[message_handler]
async fn handle_obj_click(player: &mut Player, msg: ObjClick) {
    if crate::player::is_action_locked(player) {
        return;
    }

    player.world().action_states.lock().remove(&player.index);

    let (id, position) = {
        let world = player.world();
        let Some(id) = world
            .obj_stacks
            .find(msg.item_id, msg.x as i32, msg.y as i32, player.index)
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
        .set(InteractionTarget::ObjStack { id, position }, ClickOption::One);

    with_movement!(player, |m, ctx| m
        .walk_to(&mut ctx, position, msg.force_run, None)
        .await);
}

pub fn pickup_obj_stack(target: InteractionTarget) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    let InteractionTarget::ObjStack { id, position } = target else { unreachable!() };

    Box::pin(async move {
        let player = crate::player::active_player();
        let (item_id, amount, owner, private_ticks_remaining, public_ticks_remaining, other_indices) = {
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
                snap.item_id,
                snap.amount,
                snap.owner,
                snap.private_ticks_remaining,
                snap.public_ticks_remaining,
                other_indices,
            )
        };

        player.obj_stack_mut().forget(id, item_id, position).await;

        for index in other_indices {
            let world = player.world();
            let mut p = world.players.get_mut(index);
            p.obj_stack_mut().forget(id, item_id, position).await;
        }

        let remainder = player.inv_mut().add(item_id, amount).await;

        if remainder > 0 {
            player.world().obj_stacks.add_with_state(
                item_id,
                remainder,
                position,
                owner,
                private_ticks_remaining,
                public_ticks_remaining,
            );

            player
                .outbox
                .write(ChatMessage {
                    msg_type: 0,
                    text: "You can't carry any more of that.".to_string(),
                })
                .await;
        }
    })
}
