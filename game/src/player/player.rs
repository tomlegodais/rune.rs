use crate::account::Account;
use crate::player::{Connection, Scene};
use crate::world::{Position, RegionId};
use net::{GameMessage, MessageType};
use tokio_util::bytes::{BufMut, BytesMut};
use tracing::info;
use util::{BitsMut, BytesMutExt};

pub struct Player {
    pub id: u16,
    pub _account_id: i64,
    pub username: String,
    pub _rights: u8,

    pub connection: Connection,
    pub position: Position,
    pub current_region: RegionId,
    pub scene: Scene,
}

impl Player {
    pub fn new(id: u16, account: &Account, connection: Connection, position: Position) -> Self {
        let scene = Scene::new(position, 0);
        let current_region = position.region_id();

        Self {
            id,
            _account_id: account.id,
            username: account.username.clone(),
            _rights: account.rights,
            connection,
            position,
            current_region,
            scene,
        }
    }

    pub async fn on_login(&mut self) {
        self.send_game_scene(true).await;
        self.send_root_interface(746).await;

        info!("Player ({}) logged in", self.username);
    }

    pub async fn tick(&mut self) {
        if self.scene.needs_rebuild(self.position) {
            self.scene.rebuild(self.position);
            self.send_game_scene(false).await;
        }
    }

    async fn send_game_scene(&mut self, init: bool) {
        let mut buf = BytesMut::new();

        if init {
            let mut bit_pos = buf.bits_start();
            buf.put_bits(&mut bit_pos, 30, self.position.bits());

            for _player_index in 1..2048 {
                if _player_index == self.id {
                    continue;
                }
                buf.put_bits(&mut bit_pos, 18, 0);
            }

            buf.bits_end(bit_pos);
        }

        buf.put_u8_sub(self.scene.size);
        buf.put_u16_add(self.scene.center_chunk_x as u16);
        buf.put_u16_le_add(self.scene.center_chunk_y as u16);
        buf.put_u8_add(0);

        for _region_id in &self.scene.region_ids {
            for _ in 0..4 {
                buf.put_u32(0);
            }
        }

        let msg = GameMessage {
            opcode: 13,
            ty: MessageType::Short,
            payload: buf.freeze(),
        };

        let _ = self.connection.outbound.send(msg).await;
    }

    async fn send_root_interface(&mut self, root_id: u16) {
        let mut buf = BytesMut::new();
        buf.put_u8(0);
        buf.put_u16_le(0);
        buf.put_u16_le(root_id);

        let msg = GameMessage {
            opcode: 102,
            ty: MessageType::Fixed,
            payload: buf.freeze(),
        };

        let _ = self.connection.outbound.send(msg).await;
    }
}
