use std::collections::HashMap;

use parking_lot::Mutex;

use crate::world::Position;

pub struct ObjStack {
    pub id: u32,
    pub obj_id: u16,
    pub amount: u32,
    pub position: Position,
    pub owner: Option<usize>,
    pub private_ticks_remaining: u16,
    pub public_ticks_remaining: u16,
}

pub struct ObjStackSnapshot {
    pub obj_id: u16,
    pub amount: u32,
    pub position: Position,
    pub owner: Option<usize>,
    pub private_ticks_remaining: u16,
    pub public_ticks_remaining: u16,
}

#[derive(Default)]
pub struct ObjStackStore {
    inner: Mutex<StoreInner>,
}

#[derive(Default)]
struct StoreInner {
    items: HashMap<u32, ObjStack>,
    next_id: u32,
}

impl ObjStackStore {
    pub fn add(&self, obj_id: u16, amount: u32, position: Position, owner: Option<usize>) -> u32 {
        let mut inner = self.inner.lock();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        inner.items.insert(
            id,
            ObjStack {
                id,
                obj_id,
                amount,
                position,
                owner,
                private_ticks_remaining: 100,
                public_ticks_remaining: 200,
            },
        );
        id
    }

    pub fn remove(&self, id: u32) -> Option<ObjStack> {
        self.inner.lock().items.remove(&id)
    }

    pub fn find(&self, obj_id: u16, x: i32, y: i32, viewer: usize) -> Option<u32> {
        let inner = self.inner.lock();
        inner
            .items
            .values()
            .find(|g| {
                g.obj_id == obj_id
                    && g.position.x == x
                    && g.position.y == y
                    && (g.owner.is_none() || g.owner == Some(viewer))
            })
            .map(|g| g.id)
    }

    pub fn with_items<F>(&self, mut f: F)
    where
        F: FnMut(&mut ObjStack),
    {
        let mut inner = self.inner.lock();
        for item in inner.items.values_mut() {
            f(item);
        }
    }

    pub fn decay(&self) -> Vec<ObjStack> {
        let mut expired_ids = Vec::new();

        self.with_items(|item| {
            if item.private_ticks_remaining > 0 {
                item.private_ticks_remaining -= 1;
                if item.private_ticks_remaining == 0 {
                    item.owner = None;
                }
            } else {
                item.public_ticks_remaining = item.public_ticks_remaining.saturating_sub(1);
                if item.public_ticks_remaining == 0 {
                    expired_ids.push(item.id);
                }
            }
        });

        expired_ids.into_iter().filter_map(|id| self.remove(id)).collect()
    }

    pub fn visible_to(
        &self,
        player_index: usize,
        viewport_contains: impl Fn(Position) -> bool,
    ) -> Vec<(u32, u16, u32, Position)> {
        let inner = self.inner.lock();
        inner
            .items
            .values()
            .filter(|g| viewport_contains(g.position) && (g.owner.is_none() || g.owner == Some(player_index)))
            .map(|g| (g.id, g.obj_id, g.amount, g.position))
            .collect()
    }

    pub fn get(&self, id: u32) -> Option<ObjStackSnapshot> {
        let inner = self.inner.lock();
        inner.items.get(&id).map(|g| ObjStackSnapshot {
            obj_id: g.obj_id,
            amount: g.amount,
            position: g.position,
            owner: g.owner,
            private_ticks_remaining: g.private_ticks_remaining,
            public_ticks_remaining: g.public_ticks_remaining,
        })
    }

    pub fn add_with_state(
        &self,
        obj_id: u16,
        amount: u32,
        position: Position,
        owner: Option<usize>,
        private_ticks_remaining: u16,
        public_ticks_remaining: u16,
    ) -> u32 {
        let mut inner = self.inner.lock();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        inner.items.insert(
            id,
            ObjStack {
                id,
                obj_id,
                amount,
                position,
                owner,
                private_ticks_remaining,
                public_ticks_remaining,
            },
        );
        id
    }
}
