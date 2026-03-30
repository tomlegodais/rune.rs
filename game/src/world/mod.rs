mod collision;
mod grounditem;
mod pathfinding;
mod position;
mod slab;
mod tick;

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, Weak},
};

pub use collision::CollisionMap;
pub use grounditem::GroundItemStore;
use net::{Frame, IncomingMessage};
use parking_lot::Mutex;
pub use pathfinding::{can_interact_rect, find_path, find_path_adjacent_rect};
use persistence::{account::Account, player::PlayerData};
pub use position::{Direction, Position, RegionId, Teleport, running_direction};
pub use slab::WorldSlab;
use tokio::sync::mpsc;
use tracing::info;

use crate::{
    npc::{Npc, NpcSnapshot},
    player::{ActionState, Player, PlayerSnapshot},
    world::slab::{SlabReadGuard, SlabWriteGuard},
};

pub struct World {
    self_ref: OnceLock<Weak<World>>,
    pub players: WorldSlab<Player>,
    pub npcs: WorldSlab<Npc>,
    pub ground_items: GroundItemStore,
    pub action_states: Mutex<HashMap<usize, ActionState>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            self_ref: OnceLock::new(),
            players: WorldSlab::new(),
            npcs: WorldSlab::new(),
            ground_items: GroundItemStore::default(),
            action_states: Mutex::new(HashMap::new()),
        }
    }
}

impl World {
    pub fn init(self: &Arc<Self>) {
        let _ = self.self_ref.set(Arc::downgrade(self));
        self.spawn_npc(2, Position::new(3093, 3495, 0), 4);
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
        let player = Player::new(
            index,
            account,
            player_data,
            inbox_rx,
            outbound_tx,
            display_mode,
            &snapshots,
        );

        self.players.insert(player);
        self.players.get_mut(index).set_world(&self.arc());

        (index, inbox_tx, outbound_rx)
    }

    pub fn unregister_player(&self, player_index: usize) -> Option<PlayerData> {
        if !self.players.contains(player_index) {
            return None;
        }

        self.action_states.lock().remove(&player_index);
        let player = self.players.remove(player_index);

        info!("Player (id={}, username={}) logged out", player.index, player.username);

        Some(player.to_player_data())
    }

    pub(super) async fn decay_ground_items(&self) {
        for item in self.ground_items.decay() {
            for index in self.players.keys() {
                self.players
                    .get_mut(index)
                    .ground_item_mut()
                    .forget(item.id, item.item_id, item.position)
                    .await;
            }
        }
    }

    pub fn spawn_npc(&self, npc_id: u16, position: Position, wander_radius: u8) -> usize {
        let index = self.npcs.vacant_index();
        let npc = Npc::new(index, npc_id, position, wander_radius);

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
