mod message;

use net::{Encodable, Frame};
use tokio::sync::mpsc;

pub use message::scene::GameScene;
pub use message::skill::UpdateSkill;
pub use message::widget::{OpenWidget, SetRootWidget};

pub type Inbox = mpsc::Receiver<Frame>;
pub type Outbox = mpsc::Sender<Frame>;

pub trait InboxExt {
    fn try_recv_all(&mut self) -> Vec<Frame>;
}

impl InboxExt for Inbox {
    fn try_recv_all(&mut self) -> Vec<Frame> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.try_recv() {
            messages.push(msg);
        }
        messages
    }
}

#[allow(async_fn_in_trait)]
pub trait OutboxExt {
    async fn write(&mut self, msg: impl Encodable);
}

impl OutboxExt for Outbox {
    async fn write(&mut self, msg: impl Encodable) {
        let _ = self.send(msg.encode()).await;
    }
}
