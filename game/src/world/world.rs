use crate::account::Account;
use crate::player::{Player, PlayerSnapshot};
use crate::world::{Position, RegionMap};
use net::InboxExt;
use net::{Frame, IncomingMessage};
use slab::Slab;
use tokio::sync::mpsc;
use tracing::info;

pub struct World {
    pub players: Slab<Player>,
    region_map: RegionMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Slab::new(),
            region_map: RegionMap::new(),
        }
    }

    pub async fn process_messages(&mut self) {
        for (_, player) in self.players.iter_mut() {
            let messages = player.inbox.try_recv_all();
            for message in messages {
                crate::handler::handle(player, message).await;
            }
        }
    }

    pub fn register_player(
        &mut self,
        account: &Account,
        display_mode: u8,
    ) -> (usize, mpsc::Sender<IncomingMessage>, mpsc::Receiver<Frame>) {
        let (inbox_tx, inbox_rx) = mpsc::channel::<IncomingMessage>(128);
        let (outbound_tx, outbound_rx) = mpsc::channel::<Frame>(128);

        let snapshots = self.player_snapshots();
        let id = self.players.vacant_key() + 1;

        let position = if id == 1 {
            Position::default()
        } else {
            Position::new(3094, 3493, 0)
        };

        let player = Player::new(
            id,
            &account,
            inbox_rx,
            outbound_tx,
            position,
            display_mode,
            &snapshots,
        );

        let region_id = player.position.region_id();
        self.region_map.add_player(id, region_id);
        self.players.insert(player);
        (id, inbox_tx, outbound_rx)
    }

    pub fn unregister_player(&mut self, player_id: usize) {
        let key = player_id - 1;
        if self.players.contains(key) {
            let player = self.players.remove(key);
            self.region_map
                .remove_player(player_id, player.current_region);

            info!(
                "Player (id={}, username={}) logged out",
                player.id, player.username
            );
        }
    }

    pub async fn on_player_login(&mut self, player_id: usize) {
        if let Some(player) = self.players.get_mut(player_id - 1) {
            player.on_login().await;
        }
    }

    pub async fn tick(&mut self) {
        let snapshots = self.player_snapshots();
        for (_, player) in self.players.iter_mut() {
            Self::process_player_tick(player, &snapshots, &mut self.region_map).await;
        }

        for (_, player) in self.players.iter_mut() {
            player.send_player_info().await;
        }

        for (_, player) in self.players.iter_mut() {
            player.reset();
        }
    }

    async fn process_player_tick(
        player: &mut Player,
        snapshots: &[PlayerSnapshot],
        region_map: &mut RegionMap,
    ) {
        let new_region = player.position.region_id();
        if new_region != player.current_region {
            region_map.update_player_region(player.id, player.current_region, new_region);
            player.current_region = new_region;
        }

        player.tick(snapshots).await;
    }

    fn player_snapshots(&self) -> Vec<PlayerSnapshot> {
        self.players.iter().map(|(_, p)| p.snapshot()).collect()
    }
}
