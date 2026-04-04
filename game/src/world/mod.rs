mod collision;
mod loc;
mod objstack;
mod pathfinding;
mod position;
mod slab;
mod tick;

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, Weak},
};

pub use collision::{CollisionMap, LocParams};
pub use loc::{LocStore, TempLoc, TempLocSnapshot};
use net::{Frame, IncomingMessage};
pub use objstack::ObjStackStore;
use parking_lot::Mutex;
pub use pathfinding::{
    can_interact_loc, can_interact_rect, find_path, find_path_adjacent_rect, find_path_to_loc, wall_face_direction,
};
use persistence::{account::Account, player::PlayerData};
pub use position::{Direction, Position, RegionId, Teleport, running_direction};
pub use slab::WorldSlab;
use tokio::sync::mpsc;

use crate::{
    npc::{Npc, NpcSnapshot},
    player::{ActionState, Player, PlayerSnapshot},
    world::slab::{SlabReadGuard, SlabWriteGuard},
};

struct NpcRespawn {
    npc_id: u16,
    position: Position,
    wander_radius: u8,
    max_hp: u32,
    timer: u16,
}

const NPC_RESPAWN_TICKS: u16 = 30;

pub struct World {
    self_ref: OnceLock<Weak<World>>,
    pub players: WorldSlab<Player>,
    pub npcs: WorldSlab<Npc>,
    pub obj_stacks: ObjStackStore,
    pub locs: LocStore,
    pub action_states: Mutex<HashMap<usize, ActionState>>,
    npc_respawns: Mutex<Vec<NpcRespawn>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            self_ref: OnceLock::new(),
            players: WorldSlab::new(),
            npcs: WorldSlab::new(),
            obj_stacks: ObjStackStore::default(),
            locs: LocStore::default(),
            action_states: Mutex::new(HashMap::new()),
            npc_respawns: Mutex::new(Vec::new()),
        }
    }
}

impl World {
    pub fn init(self: &Arc<Self>) {
        let _ = self.self_ref.set(Arc::downgrade(self));
        self.spawn_npc(2, Position::new(3093, 3495, 0), 4, 10);
    }

    pub fn register_player(
        &self,
        account: &Account,
        player_data: &PlayerData,
        display_mode: u8,
    ) -> (usize, mpsc::Sender<IncomingMessage>, mpsc::Receiver<Frame>) {
        let (inbox_tx, inbox_rx) = mpsc::channel::<IncomingMessage>(128);
        let (outbound_tx, outbound_rx) = mpsc::channel::<Frame>(128);
        let snapshots = self.player_snapshots();
        let index = self.players.vacant_index();
        let player = Player::new(index, account, player_data, inbox_rx, outbound_tx, &snapshots);
        self.players.insert(player);

        let mut guard = self.players.get_mut(index);
        guard.set_world(&self.arc());
        guard.init_systems(player_data, display_mode);

        (index, inbox_tx, outbound_rx)
    }

    pub fn unregister_player(&self, player_index: usize) -> Option<PlayerData> {
        if !self.players.contains(player_index) {
            return None;
        }

        self.action_states.lock().remove(&player_index);
        let player = self.players.remove(player_index);

        tracing::info!(index = player.index, username = player.username, "Player Logged Out");

        Some(player.to_player_data())
    }

    pub(super) async fn decay_obj_stacks(&self) {
        for item in self.obj_stacks.decay() {
            for index in self.players.keys() {
                self.players
                    .get_mut(index)
                    .obj_stack_mut()
                    .forget(item.id, item.obj_id, item.position)
                    .await;
            }
        }
    }

    pub(super) async fn respawn_locs(&self) {
        for expired in self.locs.tick() {
            for index in self.players.keys() {
                self.players.get_mut(index).loc_mut().on_expire(&expired).await;
            }
        }
    }

    pub(super) fn process_npc_deaths(&self) {
        let dead: Vec<usize> = self
            .npcs
            .keys()
            .into_iter()
            .filter(|&i| self.npcs.get(i).is_dead())
            .collect();
        let mut respawns = self.npc_respawns.lock();
        for idx in dead {
            let npc = self.npcs.remove(idx);
            respawns.push(NpcRespawn {
                npc_id: npc.npc_id,
                position: npc.spawn_position,
                wander_radius: npc.wander_radius,
                max_hp: npc.max_hp,
                timer: NPC_RESPAWN_TICKS,
            });
        }
    }

    pub(super) fn tick_npc_respawns(&self) {
        let ready: Vec<NpcRespawn> = {
            let mut respawns = self.npc_respawns.lock();
            respawns.iter_mut().for_each(|r| r.timer -= 1);
            let (ready, pending): (Vec<_>, Vec<_>) = respawns.drain(..).partition(|r| r.timer == 0);
            *respawns = pending;
            ready
        };
        for r in ready {
            self.spawn_npc(r.npc_id, r.position, r.wander_radius, r.max_hp);
        }
    }

    pub fn spawn_npc(&self, npc_id: u16, position: Position, wander_radius: u8, max_hp: u32) -> usize {
        let index = self.npcs.vacant_index();
        let npc = Npc::new(index, npc_id, position, wander_radius, max_hp);

        self.npcs.insert(npc);
        self.npcs.get_mut(index).set_world(&self.arc());

        index
    }

    pub fn player(&self, index: usize) -> SlabReadGuard<'_, Player> {
        self.players.get(index)
    }

    pub fn player_mut(&self, index: usize) -> SlabWriteGuard<'_, Player> {
        self.players.get_mut(index)
    }

    pub fn npc(&self, index: usize) -> SlabReadGuard<'_, Npc> {
        self.npcs.get(index)
    }

    pub fn npc_mut(&self, index: usize) -> SlabWriteGuard<'_, Npc> {
        self.npcs.get_mut(index)
    }

    pub fn is_online(&self, account_id: i64) -> bool {
        self.players.any(|p| p.account_id == account_id)
    }

    pub(super) fn player_snapshots(&self) -> Vec<PlayerSnapshot> {
        self.players.map(|p| p.snapshot())
    }

    pub(super) fn npc_snapshots(&self) -> Vec<NpcSnapshot> {
        self.npcs.map(|n| n.snapshot())
    }

    pub fn arc(&self) -> Arc<World> {
        self.self_ref
            .get()
            .expect("world not initialized")
            .upgrade()
            .expect("world has been dropped")
    }
}
