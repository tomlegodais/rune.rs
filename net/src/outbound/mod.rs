mod ifclosesub;
mod ifevents;
mod ifopensub;
mod ifopentop;
mod ifsettext;
mod inv;
mod loc;
mod logout;
mod message;
mod minimap;
mod obj;
mod playerop;
mod rebuild;
mod runenergy;
mod stat;
mod varp;
mod zone;

pub use ifclosesub::IfCloseSub;
pub use ifevents::{IfEvents, IfSetEvents};
pub use ifopensub::IfOpenSub;
pub use ifopentop::IfOpenTop;
pub use ifsettext::IfSetText;
pub use inv::{InvEntry, InvType, UpdateInvFull};
pub use loc::{LocAddChange, LocDel};
pub use logout::Logout;
pub use message::MessageGame;
pub use minimap::MinimapToggle;
pub use obj::{ObjAdd, ObjDel};
pub use playerop::SetPlayerOp;
pub use rebuild::RebuildNormal;
pub use runenergy::UpdateRunEnergy;
pub use stat::UpdateStat;
use tokio::sync::mpsc;
pub use varp::{VarbitLarge, VarbitSmall, VarpLarge, VarpSmall};
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
