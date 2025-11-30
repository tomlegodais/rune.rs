use crate::account::Account;
use crate::player::{Connection, Player};
use crate::world::{Position, RegionMap};
use net::GameMessage;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct World {
    pub players: HashMap<u16, Player>,
    region_map: RegionMap,
    next_index: u16,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            region_map: RegionMap::new(),
            next_index: 1,
        }
    }

    pub fn register_player(
        &mut self,
        account: &Account,
    ) -> (u16, mpsc::Sender<GameMessage>, mpsc::Receiver<GameMessage>) {
        let (inbox_tx, inbox_rx) = mpsc::channel::<GameMessage>(128);
        let (outbound_tx, outbound_rx) = mpsc::channel::<GameMessage>(128);

        let id = self.next_index;
        self.next_index += 1;

        let connection = Connection {
            inbox: inbox_rx,
            outbound: outbound_tx,
        };

        let player = Player::new(id, &account, connection, Position::default());
        let region_id = player.position.region_id();

        self.region_map.add_player(player.id, region_id);
        self.players.insert(id, player);
        (id, inbox_tx, outbound_rx)
    }

    pub async fn on_player_login(&mut self, player_id: u16) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.on_login().await;
        }
    }

    pub async fn tick(&mut self) {
        for player in self.players.values_mut() {
            while let Ok(msg) = player.connection.inbox.try_recv() {
                Self::handle_message(player, msg).await;
            }

            Self::process_player_tick(player, &mut self.region_map).await;
        }
    }

    async fn handle_message(_player: &mut Player, _msg: GameMessage) {
        // TODO: handle incoming messages
    }

    async fn process_player_tick(player: &mut Player, region_map: &mut RegionMap) {
        let new_region = player.position.region_id();
        if new_region != player.current_region {
            region_map.update_player_region(player.id, player.current_region, new_region);
            player.current_region = new_region;
        }

        player.tick().await;
    }
}
