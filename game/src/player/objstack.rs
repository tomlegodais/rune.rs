use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use net::ZoneFrame;

use crate::{
    player::{
        Clientbound, PlayerSnapshot,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::{ObjStackStore, Position, World},
};

pub struct ObjStackManager {
    player: PlayerHandle,
    pub known: HashMap<u32, u32>,
}

impl ObjStackManager {
    pub fn drop(&self, obj_id: u16, amount: u32, pos: Position, world: &World) {
        world.obj_stacks.add(obj_id, amount, pos, Some(self.player.index));
    }

    pub async fn forget(&mut self, id: u32, obj_id: u16, pos: Position) {
        if self.known.remove(&id).is_some() {
            self.send_obj_del(obj_id, pos).await;
        }
    }

    pub async fn on_viewport_rebuild(&mut self, obj_stacks: &ObjStackStore) {
        let ids: Vec<u32> = self.known.drain().map(|(id, _)| id).collect();
        for id in ids {
            if let Some(item) = obj_stacks.get(id) {
                self.send_obj_del(item.obj_id, item.position).await;
            }
        }
    }

    fn region_base(&self) -> Position {
        self.player.viewport.region_base
    }

    async fn send_obj_add(&mut self, obj_id: u16, amount: u32, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base());
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.player.obj_add(zone_frame, obj_id, amount, packed_offset).await;
    }

    pub async fn send_obj_del(&mut self, obj_id: u16, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base());
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.player.obj_del(zone_frame, obj_id, packed_offset).await;
    }

    async fn send_obj_count(&mut self, obj_id: u16, old_amount: u32, new_amount: u32, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base());
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.player
            .obj_count(zone_frame, obj_id, old_amount, new_amount, packed_offset)
            .await;
    }
}

#[player_system]
impl PlayerSystem for ObjStackManager {
    type TickContext = Arc<World>;

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            known: HashMap::new(),
        }
    }

    fn tick_context(world: &Arc<World>, _: &PlayerSnapshot) -> Arc<World> {
        world.clone()
    }

    fn tick<'a>(&'a mut self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let player_pos = self.player.position;
            let player_index = self.player.index;
            let in_range = |pos: Position| {
                pos.plane == player_pos.plane
                    && (pos.x - player_pos.x).abs() <= 15
                    && (pos.y - player_pos.y).abs() <= 15
            };

            let visible = world.obj_stacks.visible_to(player_index, in_range);
            for (id, obj_id, amount, pos) in visible {
                match self.known.get(&id).copied() {
                    None => {
                        self.known.insert(id, amount);
                        self.send_obj_add(obj_id, amount, pos).await;
                    }
                    Some(prev) if prev != amount => {
                        self.known.insert(id, amount);
                        self.send_obj_count(obj_id, prev, amount, pos).await;
                    }
                    _ => {}
                }
            }

            let invisible: Vec<u32> = self
                .known
                .keys()
                .copied()
                .filter(|id| {
                    world
                        .obj_stacks
                        .get(*id)
                        .map(|g| !in_range(g.position) || (g.owner.is_some() && g.owner != Some(player_index)))
                        .unwrap_or(true)
                })
                .collect();

            for id in invisible {
                self.known.remove(&id);
                if let Some(item) = world.obj_stacks.get(id) {
                    self.send_obj_del(item.obj_id, item.position).await;
                }
            }
        })
    }
}
