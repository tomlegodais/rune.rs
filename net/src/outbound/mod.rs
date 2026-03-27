mod chat;
mod energy;
mod ifclosesub;
mod ifevents;
mod ifopensub;
mod ifopentop;
mod item;
mod logout;
mod minimap;
mod player_option;
mod scene;
mod skill;
mod varp;

use crate::{Encodable, Frame};
use tokio::sync::mpsc;

pub use chat::ChatMessage;
pub use energy::RunEnergy;
pub use ifclosesub::IfCloseSub;
pub use ifevents::{IfEvents, IfSetEvents};
pub use ifopensub::IfOpenSub;
pub use ifopentop::IfOpenTop;
pub use item::{ItemContainerEntry, ItemContainerId, UpdateItemContainer};
pub use logout::Logout;
pub use minimap::MinimapFlag;
pub use player_option::PlayerOption;
pub use scene::GameScene;
pub use skill::UpdateSkill;
pub use varp::{LargeVarbit, LargeVarp, SmallVarbit, SmallVarp};

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
