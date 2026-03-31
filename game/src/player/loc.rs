use std::{collections::HashSet, future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use net::{LocAddChange, LocDel, Outbox, OutboxExt, ZoneFrame};

use crate::{
    player::{
        PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem},
    },
    world::{LocStore, Position, TempLoc, TempLocSnapshot, World},
};

pub struct LocManager {
    outbox: Outbox,
    pub known: HashSet<u32>,
    region_base: Position,
}

pub struct LocTickContext {
    pub world: Arc<World>,
    pub position: Position,
    pub region_base: Position,
}

impl LocManager {
    pub async fn on_expire(&mut self, expired: &TempLoc) {
        if !self.known.remove(&expired.id) {
            return;
        }

        match expired.original {
            Some(original) => {
                self.send_loc_add_change(
                    original.id as u16,
                    original.loc_type,
                    original.rotation,
                    expired.position,
                )
                .await;
            }
            None => {
                self.send_loc_del(expired.loc_type, expired.rotation, expired.position)
                    .await;
            }
        }
    }

    pub async fn on_viewport_rebuild(&mut self, locs: &LocStore) {
        let ids: Vec<u32> = self.known.drain().collect();
        for id in ids {
            if let Some(snap) = locs.get(id) {
                self.undo_snap(&snap).await;
            }
        }
    }

    async fn apply_snap(&mut self, snap: &TempLocSnapshot) {
        match snap.active_id {
            Some(loc_id) => {
                self.send_loc_add_change(loc_id, snap.loc_type, snap.rotation, snap.position)
                    .await;
            }
            None => {
                self.send_loc_del(snap.loc_type, snap.rotation, snap.position).await;
            }
        }
    }

    async fn undo_snap(&mut self, snap: &TempLocSnapshot) {
        match snap.original {
            Some(original) => {
                self.send_loc_add_change(original.id as u16, original.loc_type, original.rotation, snap.position)
                    .await;
            }
            None => {
                self.send_loc_del(snap.loc_type, snap.rotation, snap.position).await;
            }
        }
    }

    async fn send_loc_add_change(&mut self, loc_id: u16, loc_type: u8, rotation: u8, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base);
        self.outbox
            .write(LocAddChange {
                zone_frame: ZoneFrame::new(zone_x, zone_y, pos.plane as u8),
                loc_id,
                loc_type,
                rotation,
                packed_offset,
            })
            .await;
    }

    async fn send_loc_del(&mut self, loc_type: u8, rotation: u8, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base);
        self.outbox
            .write(LocDel {
                zone_frame: ZoneFrame::new(zone_x, zone_y, pos.plane as u8),
                loc_type,
                rotation,
                packed_offset,
            })
            .await;
    }
}

#[player_system]
impl PlayerSystem for LocManager {
    type TickContext = LocTickContext;

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            outbox: ctx.outbox.clone(),
            known: HashSet::new(),
            region_base: Position::default(),
        }
    }

    fn tick_context(world: &Arc<World>, player: &PlayerSnapshot) -> LocTickContext {
        LocTickContext {
            world: world.clone(),
            position: player.position,
            region_base: player.region_base,
        }
    }

    fn tick<'a>(&'a mut self, ctx: &'a LocTickContext) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.region_base = ctx.region_base;
            let player_pos = ctx.position;
            let in_range = |pos: Position| {
                pos.plane == player_pos.plane
                    && (pos.x - player_pos.x).abs() <= 15
                    && (pos.y - player_pos.y).abs() <= 15
            };

            let active = ctx.world.locs.active(in_range);
            for (id, snap) in &active {
                if self.known.insert(*id) {
                    self.apply_snap(snap).await;
                }
            }

            let invisible: Vec<u32> = self
                .known
                .iter()
                .copied()
                .filter(|id| ctx.world.locs.get(*id).map(|r| !in_range(r.position)).unwrap_or(true))
                .collect();

            for id in invisible {
                self.known.remove(&id);
                if let Some(snap) = ctx.world.locs.get(id) {
                    self.undo_snap(&snap).await;
                }
            }
        })
    }
}
