use std::{collections::HashMap, future::Future, pin::Pin};

use macros::player_system;
use net::{IfCloseSub, IfOpenSub, IfOpenTop, Outbox, OutboxExt};

use crate::{
    player::{
        PlayerSnapshot,
        system::{PlayerInitContext, PlayerSystem, SystemContext},
        ui,
    },
    world::World,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    Fixed = 1,
    Resizable = 2,
    FullScreen = 3,
}

impl DisplayMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Fixed),
            2 => Some(Self::Resizable),
            3 => Some(Self::FullScreen),
            _ => None,
        }
    }

    pub fn top_interface(&self) -> TopInterface {
        match self {
            Self::Fixed => TopInterface::FIXED,
            Self::Resizable | Self::FullScreen => TopInterface::RESIZABLE,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TopInterface(pub u16);

impl TopInterface {
    const FIXED: Self = Self(548);
    const RESIZABLE: Self = Self(746);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Parent {
    Root,
    Fixed(u16),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubInterface {
    pub interface: u16,
    pub parent: Parent,
    pub fixed_component: u16,
    pub resizable_component: u16,
    pub transparent: bool,
}

impl SubInterface {
    pub const fn new(interface: u16, fixed: u16, resizable: u16) -> Self {
        Self {
            interface,
            parent: Parent::Root,
            fixed_component: fixed,
            resizable_component: resizable,
            transparent: true,
        }
    }

    pub const fn modal(interface: u16) -> Self {
        Self {
            interface,
            parent: Parent::Root,
            fixed_component: 6,
            resizable_component: 8,
            transparent: false,
        }
    }

    pub const fn overlay(interface: u16) -> Self {
        Self {
            interface,
            parent: Parent::Root,
            fixed_component: 1,
            resizable_component: 0,
            transparent: true,
        }
    }

    pub const fn chatbox(interface: u16, component: u16) -> Self {
        Self::chatbox_split(interface, component, component)
    }

    pub const fn chatbox_split(interface: u16, fixed: u16, resizable: u16) -> Self {
        Self {
            interface,
            parent: Parent::Fixed(752),
            fixed_component: fixed,
            resizable_component: resizable,
            transparent: true,
        }
    }

    pub const fn opaque(mut self) -> Self {
        self.transparent = false;
        self
    }

    pub const fn parent(mut self, parent: u16) -> Self {
        self.parent = Parent::Fixed(parent);
        self
    }

    fn resolve_parent(&self, top: TopInterface) -> u16 {
        match self.parent {
            Parent::Root => top.0,
            Parent::Fixed(id) => id,
        }
    }

    fn component(&self, mode: DisplayMode) -> u16 {
        match mode {
            DisplayMode::Fixed => self.fixed_component,
            DisplayMode::Resizable | DisplayMode::FullScreen => self.resizable_component,
        }
    }
}

pub struct InterfaceManager {
    outbox: Outbox,
    display_mode: DisplayMode,
    top: TopInterface,
    interfaces: HashMap<u32, u16>,
}

impl InterfaceManager {
    pub fn new(outbox: Outbox, display_mode: u8) -> Self {
        let display_mode = DisplayMode::from_u8(display_mode).unwrap_or(DisplayMode::Fixed);

        Self {
            outbox,
            display_mode,
            top: display_mode.top_interface(),
            interfaces: HashMap::new(),
        }
    }

    fn resolve(&self, sub: &SubInterface) -> (u16, u16, u32) {
        let parent = sub.resolve_parent(self.top);
        let component = sub.component(self.display_mode);
        let hash = ((parent as u32) << 16) | (component as u32);
        (parent, component, hash)
    }

    pub async fn open_top(&mut self, top: TopInterface) {
        if self.top != top {
            self.top = top;
        }
        self.outbox.write(IfOpenTop(top.0)).await;
    }

    pub async fn open_sub(&mut self, sub: &SubInterface) {
        self.close_sub(sub).await;

        let (parent, component, hash) = self.resolve(sub);

        self.interfaces.insert(hash, sub.interface);
        self.outbox
            .write(IfOpenSub {
                parent,
                component,
                interface: sub.interface,
                transparent: sub.transparent,
            })
            .await;
    }

    pub async fn open_subs(&mut self, subs: impl IntoIterator<Item = &'static SubInterface>) {
        for sub in subs {
            self.open_sub(sub).await;
        }
    }

    pub async fn close_sub(&mut self, sub: &SubInterface) {
        let (parent, component, hash) = self.resolve(sub);

        if self.interfaces.remove(&hash).is_some() {
            self.outbox.write(IfCloseSub { parent, component }).await;
        }
    }

    pub fn is_open(&self, sub: &SubInterface) -> bool {
        let (_, _, hash) = self.resolve(sub);
        self.interfaces.get(&hash) == Some(&sub.interface)
    }
}

#[player_system]
impl PlayerSystem for InterfaceManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self::new(ctx.outbox.clone(), ctx.display_mode)
    }

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            self.open_top(self.top).await;
            self.open_subs(ui::tabs::interfaces()).await;
            self.open_subs(ui::orbs::interfaces()).await;
            self.open_subs(ui::chat::interfaces()).await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}
}
