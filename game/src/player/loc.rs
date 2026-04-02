use std::{collections::HashSet, future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use net::ZoneFrame;

use crate::{
    player::{
        Clientbound, PlayerSnapshot,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::{LocStore, Position, TempLoc, TempLocSnapshot, World},
};

pub struct LocManager {
    player: PlayerHandle,
    pub known: HashSet<u32>,
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

    fn region_base(&self) -> Position {
        self.player.viewport.region_base
    }

    async fn send_loc_add_change(&mut self, loc_id: u16, loc_type: u8, rotation: u8, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base());
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.player
            .loc_add_change(zone_frame, loc_id, loc_type, rotation, packed_offset)
            .await;
    }

    async fn send_loc_del(&mut self, loc_type: u8, rotation: u8, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base());
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.player.loc_del(zone_frame, loc_type, rotation, packed_offset).await;
    }
}

#[player_system]
impl PlayerSystem for LocManager {
    type TickContext = Arc<World>;

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            known: HashSet::new(),
        }
    }

    fn tick_context(world: &Arc<World>, _: &PlayerSnapshot) -> Arc<World> {
        world.clone()
    }

    fn tick<'a>(&'a mut self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let player_pos = self.player.position;
            let in_range = |pos: Position| {
                pos.plane == player_pos.plane
                    && (pos.x - player_pos.x).abs() <= 15
                    && (pos.y - player_pos.y).abs() <= 15
            };

            let active = world.locs.active(in_range);
            for (id, snap) in &active {
                if self.known.insert(*id) {
                    self.apply_snap(snap).await;
                }
            }

            let invisible: Vec<u32> = self
                .known
                .iter()
                .copied()
                .filter(|id| world.locs.get(*id).map(|r| !in_range(r.position)).unwrap_or(true))
                .collect();

            for id in invisible {
                self.known.remove(&id);
                if let Some(snap) = world.locs.get(id) {
                    self.undo_snap(&snap).await;
                }
            }
        })
    }
}
