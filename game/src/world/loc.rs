use std::collections::HashMap;

use parking_lot::Mutex;

use crate::{
    provider,
    world::{Position, collision::Loc},
};

pub struct TempLoc {
    pub id: u32,
    pub position: Position,
    pub original: Option<Loc>,
    pub active_id: Option<u16>,
    pub loc_type: u8,
    pub rotation: u8,
    pub ticks_remaining: u16,
}

pub struct TempLocSnapshot {
    pub position: Position,
    pub original: Option<Loc>,
    pub active_id: Option<u16>,
    pub loc_type: u8,
    pub rotation: u8,
}

#[derive(Default)]
pub struct LocStore {
    inner: Mutex<StoreInner>,
}

#[derive(Default)]
struct StoreInner {
    items: HashMap<u32, TempLoc>,
    next_id: u32,
}

impl LocStore {
    pub fn replace(&self, position: Position, original: Loc, replacement: Option<u16>, ticks: u16) -> Option<u32> {
        let mut inner = self.inner.lock();
        let already = inner
            .items
            .values()
            .any(|r| r.position == position && r.original.is_some_and(|o| o.id == original.id));

        if already {
            return None;
        }

        let collision = provider::get_collision();
        collision.unclip_loc(position, original.id, original.loc_type, original.rotation);
        if let Some(replace_id) = replacement {
            collision.clip_loc(position, replace_id as u32, original.loc_type, original.rotation);
        }

        Some(Self::insert(
            &mut inner,
            TempLoc {
                id: 0,
                position,
                original: Some(original),
                active_id: replacement,
                loc_type: original.loc_type,
                rotation: original.rotation,
                ticks_remaining: ticks,
            },
        ))
    }

    pub fn spawn(&self, position: Position, loc_id: u16, loc_type: u8, rotation: u8, ticks: u16) -> u32 {
        provider::get_collision().clip_loc(position, loc_id as u32, loc_type, rotation);

        let mut inner = self.inner.lock();
        Self::insert(
            &mut inner,
            TempLoc {
                id: 0,
                position,
                original: None,
                active_id: Some(loc_id),
                loc_type,
                rotation,
                ticks_remaining: ticks,
            },
        )
    }

    pub fn tick(&self) -> Vec<TempLoc> {
        let mut expired = Vec::new();
        let mut inner = self.inner.lock();
        let collision = provider::get_collision();

        inner.items.retain(|_, r| {
            r.ticks_remaining = r.ticks_remaining.saturating_sub(1);
            if r.ticks_remaining == 0 {
                if let Some(active_id) = r.active_id {
                    collision.unclip_loc(r.position, active_id as u32, r.loc_type, r.rotation);
                }
                if let Some(original) = r.original {
                    collision.clip_loc(r.position, original.id, original.loc_type, original.rotation);
                }

                expired.push(TempLoc {
                    id: r.id,
                    position: r.position,
                    original: r.original,
                    active_id: r.active_id,
                    loc_type: r.loc_type,
                    rotation: r.rotation,
                    ticks_remaining: 0,
                });
                false
            } else {
                true
            }
        });

        expired
    }

    pub fn is_replaced(&self, position: Position, loc_id: u32) -> bool {
        let inner = self.inner.lock();
        inner
            .items
            .values()
            .any(|r| r.position == position && r.original.is_some_and(|o| o.id == loc_id))
    }

    pub fn get(&self, id: u32) -> Option<TempLocSnapshot> {
        let inner = self.inner.lock();
        inner.items.get(&id).map(|r| TempLocSnapshot {
            position: r.position,
            original: r.original,
            active_id: r.active_id,
            loc_type: r.loc_type,
            rotation: r.rotation,
        })
    }

    pub fn active(&self, in_range: impl Fn(Position) -> bool) -> Vec<(u32, TempLocSnapshot)> {
        let inner = self.inner.lock();
        inner
            .items
            .values()
            .filter(|r| in_range(r.position))
            .map(|r| {
                (
                    r.id,
                    TempLocSnapshot {
                        position: r.position,
                        original: r.original,
                        active_id: r.active_id,
                        loc_type: r.loc_type,
                        rotation: r.rotation,
                    },
                )
            })
            .collect()
    }

    fn insert(inner: &mut StoreInner, mut entry: TempLoc) -> u32 {
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        entry.id = id;
        inner.items.insert(id, entry);
        id
    }
}
