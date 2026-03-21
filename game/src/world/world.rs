use crate::account::Account;
use crate::player::{Player, PlayerSnapshot};
use crate::world::{Position, RegionMap};
use net::InboxExt;
use net::{Frame, IncomingMessage};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct World {
    pub players: HashMap<usize, Player>,
    region_map: RegionMap,
    next_index: usize,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            region_map: RegionMap::new(),
            next_index: 1,
        }
    }

    pub async fn process_messages(&mut self) {
        for player in self.players.values_mut() {
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

        let id = self.next_index;
        self.next_index += 1;

        let position = if id == 1 {
            Position::default()
        } else {
            Position::new(3094, 3493, 0)
        };

        let snapshots = self.player_snapshots();
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
        self.region_map.add_player(player.id, region_id);
        self.players.insert(id, player);
        (id, inbox_tx, outbound_rx)
    }

    pub async fn on_player_login(&mut self, player_id: usize) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.on_login().await;
        }
    }

    pub async fn tick(&mut self) {
        let snapshots = self.player_snapshots();
        for player in self.players.values_mut() {
            Self::process_player_tick(player, &snapshots, &mut self.region_map).await;
        }

        for player in self.players.values_mut() {
            player.send_player_info().await;
        }

        for player in self.players.values_mut() {
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
        self.players.values().map(|p| p.snapshot()).collect()
    }
}
