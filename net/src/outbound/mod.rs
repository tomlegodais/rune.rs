mod chat;
mod container;
mod energy;
mod ifclosesub;
mod ifevents;
mod ifopensub;
mod ifopentop;
mod logout;
mod minimap;
mod obj;
mod player_option;
mod scene;
mod skill;
mod varp;
mod zone;

pub use chat::ChatMessage;
pub use container::{ItemContainerEntry, ItemContainerId, UpdateItemContainer};
pub use energy::RunEnergy;
pub use ifclosesub::IfCloseSub;
pub use ifevents::{IfEvents, IfSetEvents};
pub use ifopensub::IfOpenSub;
pub use ifopentop::IfOpenTop;
pub use logout::Logout;
pub use minimap::MinimapFlag;
pub use obj::{ObjAdd, ObjDel};
pub use player_option::PlayerOption;
pub use scene::GameScene;
pub use skill::UpdateSkill;
use tokio::sync::mpsc;
pub use varp::{LargeVarbit, LargeVarp, SmallVarbit, SmallVarp};
pub use zone::ZoneFrame;

use crate::{Encodable, Frame};

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
