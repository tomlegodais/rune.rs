use net::{GameMessage, ServerMessage};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub struct Connection {
    pub inbox: mpsc::Receiver<GameMessage>,
    pub outbound: mpsc::Sender<GameMessage>,
}

impl Connection {
    pub fn drain(&mut self) -> Vec<GameMessage> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.inbox.try_recv() {
            messages.push(msg);
        }

        messages
    }

    pub async fn send(&mut self, msg: impl ServerMessage) {
        let game_msg = msg.into_game_message();
        let _ = self.outbound.send(game_msg).await;
    }
}

pub type SharedConnection = Arc<Mutex<Connection>>;
