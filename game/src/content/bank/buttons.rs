use net::Op;

use super::{LAST_X_VARP, open::send_bank_size, tab_from_component};
use crate::player::{BankPendingX, Clientbound, Player, STACK_MAX};

#[macros::on_interface(interface = 762, component = 15, op = Op1)]
async fn toggle_insert_mode() {
    player.bank_mut().toggle_insert_mode().await;
}

#[macros::on_interface(interface = 762, component = 19, op = Op1)]
async fn toggle_withdraw_notes() {
    player.bank_mut().toggle_withdraw_notes();
}

#[macros::on_interface(interface = 762, component = 27, op = Op1)]
async fn deposit_inventory() {
    player.bank_mut().deposit_all_inv().await;
    send_bank_size(&mut player).await;
}

#[macros::on_interface(interface = 762, component = 29, op = Op1)]
async fn deposit_equipment() {
    player.bank_mut().deposit_all_worn().await;
    send_bank_size(&mut player).await;
}

#[macros::on_interface(interface = 762, op = Op1)]
async fn bank_tab_switch() {
    if let Some(tab) = tab_from_component(component) {
        player.bank_mut().set_current_tab(tab).await;
    }
}

#[macros::on_interface(interface = 762, op = Op2)]
async fn bank_tab_collapse() {
    if let Some(tab) = tab_from_component(component) {
        player.bank_mut().collapse_tab(tab).await;
        send_bank_size(&mut player).await;
    }
}

#[macros::on_interface(interface = 762, component = 87)]
async fn bank_withdraw() {
    let slot = slot1 as usize;
    match op {
        Op::Op1 => withdraw(&mut player, slot, 1).await,
        Op::Op2 => withdraw(&mut player, slot, 5).await,
        Op::Op3 => withdraw(&mut player, slot, 10).await,
        Op::Op4 => {
            let amount = player.bank().last_x();
            withdraw(&mut player, slot, amount).await;
        }
        Op::Op5 => prompt_x(&mut player, slot, true).await,
        Op::Op6 => {
            let Some(obj) = player.bank().slot(slot) else { return };
            withdraw(&mut player, slot, obj.amount.saturating_sub(1)).await;
        }
        Op::Op8 => examine_bank(&mut player, slot).await,
        Op::Op9 => withdraw(&mut player, slot, STACK_MAX).await,
        _ => {}
    }
}

#[macros::on_interface(interface = 763, component = 0)]
async fn bank_deposit() {
    let slot = slot1 as usize;
    match op {
        Op::Op1 => deposit(&mut player, slot, 1).await,
        Op::Op2 => deposit(&mut player, slot, 5).await,
        Op::Op3 => deposit(&mut player, slot, 10).await,
        Op::Op4 => {
            let amount = player.bank().last_x();
            deposit(&mut player, slot, amount).await;
        }
        Op::Op5 => prompt_x(&mut player, slot, false).await,
        Op::Op8 => examine_inv(&mut player, slot).await,
        Op::Op9 => deposit(&mut player, slot, STACK_MAX).await,
        _ => {}
    }
}

async fn withdraw(player: &mut Player, slot: usize, amount: u32) {
    if amount == 0 {
        return;
    }
    player.bank_mut().withdraw(slot, amount).await;
    send_bank_size(player).await;
}

async fn deposit(player: &mut Player, slot: usize, amount: u32) {
    if amount == 0 {
        return;
    }
    player.bank_mut().deposit(slot, amount).await;
    send_bank_size(player).await;
}

async fn examine_bank(player: &mut Player, slot: usize) {
    let Some(obj) = player.bank().slot(slot) else { return };
    send_examine(player, obj.id).await;
}

async fn examine_inv(player: &mut Player, slot: usize) {
    let Some(obj) = player.inv().slot(slot) else { return };
    send_examine(player, obj.id).await;
}

async fn send_examine(player: &mut Player, obj_id: u16) {
    let name = crate::provider::get_obj_type(obj_id as u32)
        .map(|t| t.name.as_str())
        .unwrap_or("null");
    player.send_message(format!("It's a {}.", name)).await;
}

async fn prompt_x(player: &mut Player, slot: usize, withdraw: bool) {
    player.bank_mut().set_pending_x(BankPendingX { slot, withdraw });
    player
        .count_prompt_mut()
        .prompt("Enter Amount:", |p: &mut Player, v| Box::pin(resume_count(p, v)))
        .await;
}

async fn resume_count(player: &mut Player, value: u32) {
    let Some(pending) = player.bank_mut().take_pending_x() else { return };
    if value == 0 {
        return;
    }
    player.bank_mut().set_last_x(value);
    player.varp_mut().send_varp(LAST_X_VARP, value as i32).await;
    if pending.withdraw {
        withdraw(player, pending.slot, value).await;
    } else {
        deposit(player, pending.slot, value).await;
    }
}
