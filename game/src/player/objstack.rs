use std::{collections::HashSet, future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use net::{ObjAdd, ObjDel, Outbox, OutboxExt, ZoneFrame};

use crate::{
    player::{
        PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem},
    },
    world::{ObjStackStore, Position, World},
};

pub struct ObjStackManager {
    player_index: usize,
    outbox: Outbox,
    pub known: HashSet<u32>,
    region_base: Position,
}

pub struct ObjStackTickContext {
    pub world: Arc<World>,
    pub position: Position,
    pub region_base: Position,
}

impl ObjStackManager {
    pub fn drop(&self, item_id: u16, amount: u32, pos: Position, world: &World) {
        world.obj_stacks.add(item_id, amount, pos, Some(self.player_index));
    }

    pub async fn forget(&mut self, id: u32, item_id: u16, pos: Position) {
        if self.known.remove(&id) {
            self.send_objdel(item_id, pos).await;
        }
    }

    pub async fn on_viewport_rebuild(&mut self, obj_stacks: &ObjStackStore) {
        let ids: Vec<u32> = self.known.drain().collect();
        for id in ids {
            if let Some(item) = obj_stacks.get(id) {
                self.send_objdel(item.item_id, item.position).await;
            }
        }
    }

    async fn send_objadd(&mut self, item_id: u16, amount: u32, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base);
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.outbox
            .write(ObjAdd {
                zone_frame,
                item_id,
                amount,
                packed_offset,
            })
            .await;
    }

    pub async fn send_objdel(&mut self, item_id: u16, pos: Position) {
        let (zone_x, zone_y, packed_offset) = pos.zone_coords(self.region_base);
        let zone_frame = ZoneFrame::new(zone_x, zone_y, pos.plane as u8);
        self.outbox
            .write(ObjDel {
                zone_frame,
                item_id,
                packed_offset,
            })
            .await;
    }
}

#[player_system]
impl PlayerSystem for ObjStackManager {
    type TickContext = ObjStackTickContext;

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player_index: ctx.index,
            outbox: ctx.outbox.clone(),
            known: HashSet::new(),
            region_base: Position::default(),
        }
    }

    fn tick_context(world: &Arc<World>, player: &PlayerSnapshot) -> ObjStackTickContext {
        ObjStackTickContext {
            world: world.clone(),
            position: player.position,
            region_base: player.region_base,
        }
    }

    fn tick<'a>(&'a mut self, ctx: &'a ObjStackTickContext) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.region_base = ctx.region_base;
            let player_pos = ctx.position;
            let player_index = self.player_index;
            let in_range = |pos: Position| {
                pos.plane == player_pos.plane
                    && (pos.x - player_pos.x).abs() <= 15
                    && (pos.y - player_pos.y).abs() <= 15
            };

            let visible = ctx.world.obj_stacks.visible_to(player_index, in_range);
            for (id, item_id, amount, pos) in visible {
                if self.known.insert(id) {
                    self.send_objadd(item_id, amount, pos).await;
                }
            }

            let invisible: Vec<u32> = self
                .known
                .iter()
                .copied()
                .filter(|id| {
                    ctx.world
                        .obj_stacks
                        .get(*id)
                        .map(|g| !in_range(g.position) || (g.owner.is_some() && g.owner != Some(player_index)))
                        .unwrap_or(true)
                })
                .collect();

            for id in invisible {
                self.known.remove(&id);
                if let Some(item) = ctx.world.obj_stacks.get(id) {
                    self.send_objdel(item.item_id, item.position).await;
                }
            }
        })
    }
}
