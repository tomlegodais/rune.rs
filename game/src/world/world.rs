use crate::player::{Player, PlayerSnapshot};
use crate::world::RegionMap;
use net::{Frame, IncomingMessage};
use persistence::account::Account;
use persistence::player::PlayerData;
use slab::Slab;
use tokio::sync::mpsc;
use tracing::info;

#[derive(Default)]
pub struct World {
    pub players: Slab<Player>,
    pub(super) region_map: RegionMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Slab::new(),
            region_map: RegionMap::new(),
        }
    }

    pub fn register_player(
        &mut self,
        account: &Account,
        player_data: &PlayerData,
        display_mode: u8,
    ) -> (usize, mpsc::Sender<IncomingMessage>, mpsc::Receiver<Frame>) {
        let (inbox_tx, inbox_rx) = mpsc::channel::<IncomingMessage>(128);
        let (outbound_tx, outbound_rx) = mpsc::channel::<Frame>(128);
        let snapshots = self.player_snapshots();
        let id = self.players.vacant_key() + 1;
        let player = Player::new(
            id,
            account,
            player_data,
            inbox_rx,
            outbound_tx,
            display_mode,
            &snapshots,
        );

        let region_id = player.position.region_id();
        self.region_map.add_player(id, region_id);
        self.players.insert(player);
        (id, inbox_tx, outbound_rx)
    }

    pub fn unregister_player(&mut self, player_id: usize) -> Option<PlayerData> {
        let key = player_id - 1;
        if !self.players.contains(key) {
            return None;
        }

        let player = self.players.remove(key);
        self.region_map
            .remove_player(player_id, player.current_region);

        info!(
            "Player (id={}, username={}) logged out",
            player.id, player.username
        );

        Some(player.to_player_data())
    }

    pub async fn on_player_login(&mut self, player_id: usize) {
        if let Some(player) = self.players.get_mut(player_id - 1) {
            player.on_login().await;
        }
    }

    pub fn is_online(&self, account_id: i64) -> bool {
        self.players
            .iter()
            .any(|(_, p)| p._account_id == account_id)
    }

    pub(super) fn player_snapshots(&self) -> Vec<PlayerSnapshot> {
        self.players.iter().map(|(_, p)| p.snapshot()).collect()
    }
}
