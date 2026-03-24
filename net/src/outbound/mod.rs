mod chat;
mod energy;
mod minimap;
mod scene;
mod skill;
mod varp;
mod widget;

use crate::{Encodable, Frame};
use tokio::sync::mpsc;

pub use chat::ChatMessage;
pub use energy::RunEnergy;
pub use minimap::MinimapFlag;
pub use scene::GameScene;
pub use skill::UpdateSkill;
pub use varp::{LargeVarbit, LargeVarp, SmallVarbit, SmallVarp};
pub use widget::{OpenWidget, SetRootWidget};

pub type Outbox = mpsc::Sender<Frame>;

#[allow(async_fn_in_trait)]
pub trait OutboxExt {
    async fn write(&mut self, msg: impl Encodable);
}

impl OutboxExt for Outbox {
    async fn write(&mut self, msg: impl Encodable) {
        let _ = self.send(msg.encode()).await;
    }
}
