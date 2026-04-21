use filesystem::WearPos;
use macros::player_system;
use net::{InvEntry, InvType};
use num_enum::TryFromPrimitive;
use persistence::PlayerData;

use crate::{
    player::{
        Clientbound, INV_SIZE, Obj, PlayerSnapshot, STACK_MAX, WORN_SIZE,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    provider,
    world::World,
};

pub const SIZE: usize = 516;
pub const TAB_COUNT: usize = 9;

#[derive(Clone, Copy)]
pub struct PendingX {
    pub slot: usize,
    pub withdraw: bool,
}

pub struct Bank {
    player: PlayerHandle,
    tabs: [Vec<Obj>; TAB_COUNT],
    current_tab: u8,
    last_x: u32,
    pending_x: Option<PendingX>,
    insert_mode: bool,
    withdraw_notes: bool,
}

impl Bank {
    pub fn slot(&self, fake: usize) -> Option<Obj> {
        let (tab, idx) = self.real_slot(fake)?;
        self.tabs[tab as usize].get(idx).copied()
    }

    pub fn tab_size(&self, tab: u8) -> usize {
        self.tabs.get(tab as usize).map(Vec::len).unwrap_or(0)
    }

    pub fn total_size(&self) -> usize {
        self.tabs.iter().map(Vec::len).sum()
    }

    pub fn current_tab(&self) -> u8 {
        self.current_tab
    }

    pub fn last_x(&self) -> u32 {
        self.last_x
    }

    pub fn set_last_x(&mut self, value: u32) {
        self.last_x = value;
    }

    pub fn set_pending_x(&mut self, pending: PendingX) {
        self.pending_x = Some(pending);
    }

    pub fn take_pending_x(&mut self) -> Option<PendingX> {
        self.pending_x.take()
    }

    pub fn insert_mode(&self) -> bool {
        self.insert_mode
    }

    pub async fn toggle_insert_mode(&mut self) {
        self.insert_mode = !self.insert_mode;
        self.player.varp_mut().send_varp(305, self.insert_mode as i32).await;
    }

    pub fn toggle_withdraw_notes(&mut self) {
        self.withdraw_notes = !self.withdraw_notes;
    }

    pub async fn set_current_tab(&mut self, tab: u8) {
        if tab as usize >= TAB_COUNT {
            return;
        }
        self.current_tab = tab;
        self.send_current_tab().await;
    }

    pub fn count(&self, obj_id: u16) -> u32 {
        self.tabs
            .iter()
            .flat_map(|t| t.iter())
            .filter(|o| o.id == obj_id)
            .map(|o| o.amount)
            .sum()
    }

    pub async fn swap(&mut self, from_fake: usize, to_fake: usize) {
        if from_fake == to_fake {
            return;
        }
        let Some((a_tab, a_idx)) = self.real_slot(from_fake) else { return };
        let Some((b_tab, b_idx)) = self.real_slot(to_fake) else { return };
        if a_tab == b_tab {
            self.tabs[a_tab as usize].swap(a_idx, b_idx);
        } else {
            let item_a = self.tabs[a_tab as usize][a_idx];
            let item_b = self.tabs[b_tab as usize][b_idx];
            self.tabs[a_tab as usize][a_idx] = item_b;
            self.tabs[b_tab as usize][b_idx] = item_a;
        }
        self.flush().await;
    }

    pub async fn insert(&mut self, from_fake: usize, to_fake: usize) {
        if from_fake == to_fake {
            return;
        }
        let Some((a_tab, a_idx)) = self.real_slot(from_fake) else { return };
        let Some((b_tab, b_idx)) = self.real_slot(to_fake) else { return };
        if a_tab != b_tab {
            let item = self.tabs[a_tab as usize].remove(a_idx);
            let dest_len = self.tabs[b_tab as usize].len();
            let dest_idx = b_idx.min(dest_len);
            self.tabs[b_tab as usize].insert(dest_idx, item);
            if a_tab != 0 && self.tabs[a_tab as usize].is_empty() {
                self.shift_tabs_down(a_tab as usize);
            }
        } else {
            let tab = &mut self.tabs[a_tab as usize];
            let item = tab.remove(a_idx);
            tab.insert(b_idx, item);
        }
        self.flush().await;
    }

    pub async fn move_to_tab(&mut self, from_fake: usize, target_tab: u8) {
        if target_tab as usize >= TAB_COUNT {
            return;
        }
        let Some((src_tab, src_idx)) = self.real_slot(from_fake) else { return };
        if src_tab == target_tab {
            return;
        }
        let item = self.tabs[src_tab as usize].remove(src_idx);
        self.tabs[target_tab as usize].push(item);
        if src_tab != 0 && self.tabs[src_tab as usize].is_empty() {
            self.shift_tabs_down(src_tab as usize);
        }
        self.flush().await;
    }

    pub async fn collapse_tab(&mut self, tab: u8) {
        if tab == 0 || tab as usize >= TAB_COUNT || self.tabs[tab as usize].is_empty() {
            return;
        }
        let items = std::mem::take(&mut self.tabs[tab as usize]);
        self.tabs[0].extend(items);
        self.shift_tabs_down(tab as usize);
        self.flush().await;
    }

    fn shift_tabs_down(&mut self, start: usize) {
        for i in start..TAB_COUNT - 1 {
            self.tabs[i] = std::mem::take(&mut self.tabs[i + 1]);
        }
        self.tabs[TAB_COUNT - 1] = Vec::new();
        if self.current_tab as usize == start {
            self.current_tab = 0;
        } else if self.current_tab as usize > start {
            self.current_tab -= 1;
        }
    }

    pub async fn deposit_all_inv(&mut self) {
        let mut any = false;
        for i in 0..INV_SIZE {
            let Some(clicked) = self.player.inv().slot(i) else { continue };
            let store_id = unnote(clicked.id);
            let total = self.player.inv().count(clicked.id);
            let added = self.add_stack(store_id, total);
            if added == 0 {
                continue;
            }
            self.player.inv_mut().remove(clicked.id, added).await;
            any = true;
        }
        if any {
            self.flush().await;
        }
        if (0..INV_SIZE).any(|i| self.player.inv().slot(i).is_some()) {
            crate::send_message!(&mut *self.player, "Not enough space in your bank.");
        }
    }

    pub async fn deposit_all_worn(&mut self) {
        let mut any = false;
        let mut weapon_removed = false;

        for i in 0..WORN_SIZE {
            let Ok(pos) = WearPos::try_from_primitive(i) else { continue };
            let Some(obj) = self.player.worn().slot(pos) else { continue };
            let store_id = unnote(obj.id);
            let added = self.add_stack(store_id, obj.amount);
            if added < obj.amount {
                break;
            }
            self.player.worn_mut().set(pos, None);
            any = true;
            if pos == WearPos::Weapon {
                weapon_removed = true;
            }
        }

        if any {
            self.player.worn_mut().flush().await;
            self.player.appearance_mut().flush();
            self.flush().await;
            if weapon_removed {
                self.player.combat_mut().set_combat_style(0);
                self.player.varp_mut().send_varp(43, 0).await;
            }
        }

        if (0..WORN_SIZE)
            .filter_map(|i| WearPos::try_from_primitive(i).ok())
            .any(|p| self.player.worn().slot(p).is_some())
        {
            crate::send_message!(&mut *self.player, "Not enough space in your bank.");
        }
    }

    pub async fn deposit(&mut self, inv_slot: usize, amount: u32) -> bool {
        let Some(clicked) = self.player.inv().slot(inv_slot) else {
            return false;
        };
        let store_id = unnote(clicked.id);
        let total = self.player.inv().count(clicked.id);
        let taken = amount.min(total);
        if taken == 0 {
            return false;
        }

        let added = self.add_stack(store_id, taken);
        if added == 0 {
            return false;
        }

        self.player.inv_mut().remove(clicked.id, added).await;
        self.flush().await;
        true
    }

    pub async fn withdraw(&mut self, bank_slot: usize, amount: u32) -> bool {
        let Some(obj) = self.slot(bank_slot) else { return false };
        let withdraw_id = self.resolve_withdraw_id(obj.id);
        let stackable = is_stackable(withdraw_id);
        let free = self.player.inv().free_slots() as u32;
        let has_existing_stack = stackable && self.player.inv().count(withdraw_id) > 0;

        let requested = amount.min(obj.amount);
        let capacity = match (stackable, has_existing_stack) {
            (true, true) => STACK_MAX - self.player.inv().count(withdraw_id),
            (true, false) => {
                if free > 0 {
                    STACK_MAX
                } else {
                    0
                }
            }
            (false, _) => free,
        };

        let taken = requested.min(capacity);
        if taken == 0 {
            crate::send_message!(&mut *self.player, "Not enough inventory space.");
            return false;
        }

        self.remove_at(bank_slot, taken);
        self.player.inv_mut().add(withdraw_id, taken).await;
        self.flush().await;
        true
    }

    fn resolve_withdraw_id(&mut self, obj_id: u16) -> u16 {
        if !self.withdraw_notes {
            return obj_id;
        }
        let Some(t) = provider::get_obj_type(obj_id as u32) else { return obj_id };
        if t.noted_template.is_some() {
            return obj_id;
        }
        match t.noted_id {
            Some(noted) => noted as u16,
            None => {
                crate::send_message!(&mut *self.player, "You cannot withdraw this item as a note.");
                obj_id
            }
        }
    }

    fn add_stack(&mut self, obj_id: u16, amount: u32) -> u32 {
        for tab in self.tabs.iter_mut() {
            if let Some(obj) = tab.iter_mut().find(|o| o.id == obj_id) {
                let added = amount.min(STACK_MAX - obj.amount);
                obj.amount += added;
                return added;
            }
        }
        if self.total_size() >= SIZE {
            return 0;
        }
        let added = amount.min(STACK_MAX);
        self.tabs[self.current_tab as usize].push(Obj::new(obj_id, added));
        added
    }

    fn remove_at(&mut self, fake_slot: usize, amount: u32) {
        let Some((tab, idx)) = self.real_slot(fake_slot) else { return };
        let tab_idx = tab as usize;
        let tab_vec = &mut self.tabs[tab_idx];
        if amount < tab_vec[idx].amount {
            tab_vec[idx].amount -= amount;
            return;
        }
        tab_vec.remove(idx);
        if tab != 0 && tab_vec.is_empty() {
            self.shift_tabs_down(tab_idx);
        }
    }

    fn real_slot(&self, fake: usize) -> Option<(u8, usize)> {
        let mut remaining = fake;
        for tab in 1..TAB_COUNT {
            let len = self.tabs[tab].len();
            if remaining < len {
                return Some((tab as u8, remaining));
            }
            remaining -= len;
        }
        (remaining < self.tabs[0].len()).then_some((0, remaining))
    }

    pub async fn flush(&mut self) {
        self.flush_items().await;
        self.send_tab_sizes().await;
        self.send_current_tab().await;
    }

    async fn flush_items(&mut self) {
        let mut objs = Vec::with_capacity(SIZE);
        for tab in 1..TAB_COUNT {
            for obj in &self.tabs[tab] {
                objs.push(Some(InvEntry {
                    obj_id: obj.id,
                    amount: obj.amount,
                }));
            }
        }
        for obj in &self.tabs[0] {
            objs.push(Some(InvEntry {
                obj_id: obj.id,
                amount: obj.amount,
            }));
        }
        self.player.update_inv(InvType::Bank, false, objs).await;
    }

    async fn send_tab_sizes(&mut self) {
        for tab in 1..TAB_COUNT {
            let size = self.tabs[tab].len() as i32;
            self.player
                .varp_mut()
                .send_varbit(bank_varbits::TAB_SIZE_BASE + (tab as u32 - 1), size)
                .await;
        }
    }

    pub async fn send_current_tab(&mut self) {
        self.player
            .varp_mut()
            .send_varbit(bank_varbits::CURRENT_TAB, (self.current_tab + 1) as i32)
            .await;
    }
}

pub(crate) mod bank_varbits {
    pub const CURRENT_TAB: u32 = 4893;
    pub const TAB_SIZE_BASE: u32 = 4885;
}

fn is_stackable(obj_id: u16) -> bool {
    provider::get_obj_type(obj_id as u32).is_some_and(|t| t.stackable)
}

fn unnote(obj_id: u16) -> u16 {
    match provider::get_obj_type(obj_id as u32) {
        Some(t) if t.noted_template.is_some() => t.noted_id.map(|id| id as u16).unwrap_or(obj_id),
        _ => obj_id,
    }
}

#[player_system]
impl PlayerSystem for Bank {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        let mut tabs: [Vec<Obj>; TAB_COUNT] = std::array::from_fn(|_| Vec::new());
        for (tab_idx, entries) in ctx.player_data.bank_tabs.iter().enumerate().take(TAB_COUNT) {
            tabs[tab_idx] = entries.iter().map(|(id, amount)| Obj::new(*id, *amount)).collect();
        }
        Self {
            player: ctx.player,
            tabs,
            current_tab: 0,
            last_x: ctx.player_data.bank_last_x,
            pending_x: None,
            insert_mode: false,
            withdraw_notes: false,
        }
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.bank_tabs = self
            .tabs
            .iter()
            .map(|tab| tab.iter().map(|o| (o.id, o.amount)).collect())
            .collect();
        data.bank_last_x = self.last_x;
    }
}
