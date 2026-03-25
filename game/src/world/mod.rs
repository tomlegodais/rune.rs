mod collision;
mod pathfinding;
mod position;
mod region;
mod slab;
mod tick;

pub(crate) use collision::CollisionMap;
pub(crate) use pathfinding::find_path;
pub(crate) use position::{Direction, Position, Teleport, running_direction};
pub(crate) use region::{RegionId, RegionMap};
pub(crate) use slab::WorldSlab;

use crate::npc::{Npc, NpcSnapshot};
use crate::player::{Player, PlayerSnapshot};
use net::{Frame, IncomingMessage};
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockWriteGuard};
use persistence::account::Account;
use persistence::player::PlayerData;
use std::sync::{Arc, OnceLock, Weak};
use tokio::sync::mpsc;
use tracing::info;

pub struct World {
    self_ref: OnceLock<Weak<World>>,
    pub players: WorldSlab<Player>,
    pub npcs: WorldSlab<Npc>,
    region_map: RwLock<RegionMap>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            self_ref: OnceLock::new(),
            players: WorldSlab::new(),
            npcs: WorldSlab::new(),
            region_map: RwLock::new(RegionMap::new()),
        }
    }
}

impl World {
    pub fn init(self: &Arc<Self>) {
        let _ = self.self_ref.set(Arc::downgrade(self));
        self.spawn_npc(2, Position::new(3093, 3495, 0));
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

        self.region_map()
            .add_player(index, player.position.region_id());

        self.players.insert(player);
        self.players.get_mut(index).set_world(&self.arc());

        (index, inbox_tx, outbound_rx)
    }

    pub async fn on_player_login(&self, player_index: usize) {
        self.players.get_mut(player_index).on_login().await;
    }

    pub fn unregister_player(&self, player_index: usize) -> Option<PlayerData> {
        if !self.players.contains(player_index) {
            return None;
        }

        let player = self.players.remove(player_index);
        self.region_map()
            .remove_player(player_index, player.current_region);

        info!(
            "Player (id={}, username={}) logged out",
            player.index, player.username
        );

        Some(player.to_player_data())
    }

    pub fn spawn_npc(&self, npc_id: u16, position: Position) -> usize {
        let index = self.npcs.vacant_index();
        let npc = Npc::new(index, npc_id, position);

        self.region_map().add_npc(index, npc.position.region_id());

        self.npcs.insert(npc);
        self.npcs.get_mut(index).set_world(&self.arc());

        index
    }

    pub fn player(&self, index: usize) -> MappedRwLockReadGuard<'_, Player> {
        self.players.get(index)
    }

    pub fn player_mut(&self, index: usize) -> MappedRwLockWriteGuard<'_, Player> {
        self.players.get_mut(index)
    }

    pub fn npc(&self, index: usize) -> MappedRwLockReadGuard<'_, Npc> {
        self.npcs.get(index)
    }

    pub fn npc_mut(&self, index: usize) -> MappedRwLockWriteGuard<'_, Npc> {
        self.npcs.get_mut(index)
    }

    pub fn region_map(&self) -> RwLockWriteGuard<'_, RegionMap> {
        self.region_map.write()
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

    fn arc(&self) -> Arc<World> {
        self.self_ref
            .get()
            .expect("world not initialized")
            .upgrade()
            .expect("world has been dropped")
    }
}
