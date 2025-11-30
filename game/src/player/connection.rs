use tokio::sync::mpsc;
use net::GameMessage;

pub struct Connection {
    pub inbox: mpsc::Receiver<GameMessage>,
    pub outbound: mpsc::Sender<GameMessage>,
}