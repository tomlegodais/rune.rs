use tokio::sync::mpsc;
use net::{GameMessage, ServerMessage};

pub struct Connection {
    pub inbox: mpsc::Receiver<GameMessage>,
    pub outbound: mpsc::Sender<GameMessage>,
}

impl Connection {
    pub async fn send(&mut self, msg: impl ServerMessage) {
        let game_msg = msg.into_game_message();
        let _ = self.outbound.send(game_msg).await;
    }
}