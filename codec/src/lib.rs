mod message;

pub use message::scene::GameScene;
pub use message::widget::{OpenWidget, SetRootWidget};
pub use net::{Encodable, GameMessage, MessageType};