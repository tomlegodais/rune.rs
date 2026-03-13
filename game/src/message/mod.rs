use net::GameMessage;
use tokio::sync::mpsc;

pub(crate) use codec::{Encodable, GameScene, OpenWidget, SetRootWidget};

pub(crate) type Inbox = mpsc::Receiver<GameMessage>;
pub(crate) type Outbox = mpsc::Sender<GameMessage>;

pub(crate) trait OutboxExt {
    async fn send_message(&mut self, msg: impl Encodable);
}

impl OutboxExt for Outbox {
    async fn send_message(&mut self, msg: impl Encodable) {
        let _ = self.send(msg.encode()).await;
    }
}