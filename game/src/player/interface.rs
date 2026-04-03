use std::{collections::HashMap, future::Future, pin::Pin};

use macros::{enum_data, player_system};
use num_enum::TryFromPrimitive;
use strum::IntoEnumIterator;

use crate::{
    player::{
        Clientbound, PlayerSnapshot,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
        ui,
    },
    world::World,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum DisplayMode {
    Fixed = 1,
    Resizable = 2,
    FullScreen = 3,
}

impl DisplayMode {
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
enum Parent {
    Root,
    Fixed(u16),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SubInterface {
    interface: u16,
    parent: Parent,
    fixed_component: u16,
    resizable_component: u16,
    transparent: bool,
}

impl SubInterface {
    const fn new(interface: u16, fixed: u16, resizable: u16) -> Self {
        Self {
            interface,
            parent: Parent::Root,
            fixed_component: fixed,
            resizable_component: resizable,
            transparent: true,
        }
    }

    const fn with_parent(mut self, parent: u16) -> Self {
        self.parent = Parent::Fixed(parent);
        self
    }

    fn resolve(&self, top: TopInterface, mode: DisplayMode) -> (u16, u16) {
        let parent = match self.parent {
            Parent::Root => top.0,
            Parent::Fixed(id) => id,
        };

        let component = match mode {
            DisplayMode::Fixed => self.fixed_component,
            DisplayMode::Resizable | DisplayMode::FullScreen => self.resizable_component,
        };

        (parent, component)
    }
}

#[enum_data(u16, u16, u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum TabIndex {
    Attack = (ui::tabs::ATTACK, 152, 33),
    Skills = (ui::tabs::SKILLS, 153, 34),
    Quests = (ui::tabs::QUESTS, 154, 35),
    Achievements = (ui::tabs::ACHIEVEMENTS, 155, 36),
    Inventory = (ui::tabs::INVENTORY, 156, 37),
    Equipment = (ui::tabs::EQUIPMENT, 157, 38),
    Prayer = (ui::tabs::PRAYER, 158, 39),
    Magic = (ui::tabs::MAGIC, 159, 40),
    Objective = (ui::tabs::OBJECTIVE, 160, 41),
    Friends = (ui::tabs::FRIENDS, 161, 42),
    Ignores = (ui::tabs::IGNORES, 162, 43),
    Clan = (ui::tabs::CLAN, 163, 44),
    Settings = (ui::tabs::SETTINGS, 164, 45),
    Emotes = (ui::tabs::EMOTES, 165, 46),
    Music = (ui::tabs::MUSIC, 166, 47),
    Notes = (ui::tabs::NOTES, 167, 48),
    Logout = (ui::tabs::LOGOUT, 170, 51),
}

impl TabIndex {
    pub const fn default_interface(self) -> u16 {
        self.data().0
    }
    pub const fn components(self) -> (u16, u16) {
        let d = self.data();
        (d.1, d.2)
    }
}

#[enum_data(u16, u16, u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum OrbIndex {
    Summoning = (ui::orbs::SUMMONING, 139, 172),
    Hitpoints = (ui::orbs::HITPOINTS, 134, 169),
    Prayer = (ui::orbs::PRAYER, 136, 170),
    RunEnergy = (ui::orbs::RUN_ENERGY, 137, 171),
}

impl OrbIndex {
    const fn sub(self) -> SubInterface {
        let d = self.data();
        SubInterface::new(d.0, d.1, d.2)
    }
}

#[enum_data(u16, u16, u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum ChatIndex {
    Frame = (ui::chat::FRAME, 142, 18),
    Area = (ui::chat::AREA, 9, 9),
    Options = (ui::chat::OPTIONS, 20, 15),
    PrivateArea = (ui::chat::PRIVATE_AREA, 14, 19),
}

impl ChatIndex {
    const fn sub(self) -> SubInterface {
        let d = self.data();
        let s = SubInterface::new(d.0, d.1, d.2);
        match self {
            Self::Area => s.with_parent(ui::chat::FRAME),
            _ => s,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceSlot {
    Modal,
    Overlay,
    Inventory,
    Chatbox,
    Tab(TabIndex),
}

impl InterfaceSlot {
    fn transparent(self) -> bool {
        match self {
            Self::Modal | Self::Inventory => false,
            Self::Overlay | Self::Chatbox | Self::Tab(_) => true,
        }
    }

    fn components(self) -> (u16, u16) {
        match self {
            Self::Modal => (6, 8),
            Self::Overlay => (1, 0),
            Self::Inventory => (147, 30),
            Self::Chatbox => (13, 13),
            Self::Tab(t) => t.components(),
        }
    }

    fn parent(self) -> Option<u16> {
        match self {
            Self::Chatbox => Some(ui::chat::FRAME),
            _ => None,
        }
    }

    fn resolve(self, top: TopInterface, mode: DisplayMode) -> (u16, u16) {
        let parent = self.parent().unwrap_or(top.0);
        let (fixed, resizable) = self.components();
        let component = match mode {
            DisplayMode::Fixed => fixed,
            DisplayMode::Resizable | DisplayMode::FullScreen => resizable,
        };

        (parent, component)
    }
}

pub struct InterfaceManager {
    player: PlayerHandle,
    display_mode: DisplayMode,
    top: TopInterface,
    slots: HashMap<InterfaceSlot, u16>,
}

impl InterfaceManager {
    pub async fn open_top(&mut self, top: TopInterface) {
        if self.top != top {
            self.top = top;
        }
        self.player.if_open_top(top.0).await;
    }

    pub async fn open_slot(&mut self, slot: InterfaceSlot, interface: u16) {
        let (parent, component) = slot.resolve(self.top, self.display_mode);
        if self.slots.insert(slot, interface).is_some() {
            self.player.if_close_sub(parent, component).await;
        }
        self.player
            .if_open_sub(parent, component, interface, slot.transparent())
            .await;
    }

    pub async fn close_slot(&mut self, slot: InterfaceSlot) {
        if self.slots.remove(&slot).is_some() {
            let (parent, component) = slot.resolve(self.top, self.display_mode);
            self.player.if_close_sub(parent, component).await;
        }
    }

    pub fn get_slot(&self, slot: InterfaceSlot) -> Option<u16> {
        self.slots.get(&slot).copied()
    }

    pub async fn set_text(&mut self, slot: InterfaceSlot, component: u16, text: impl Into<String> + Send) {
        let interface = match self.slots.get(&slot) {
            Some(&id) => id,
            None => return,
        };
        self.player.if_set_text(interface, component, text).await;
    }

    async fn open_sub(&mut self, sub: SubInterface) {
        let (parent, component) = sub.resolve(self.top, self.display_mode);
        self.player
            .if_open_sub(parent, component, sub.interface, sub.transparent)
            .await;
    }

    async fn open_subs(&mut self, subs: impl IntoIterator<Item = SubInterface>) {
        for sub in subs {
            self.open_sub(sub).await;
        }
    }
}

#[player_system]
impl PlayerSystem for InterfaceManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        let display_mode = DisplayMode::try_from(ctx.display_mode).unwrap_or(DisplayMode::Fixed);
        Self {
            player: ctx.player,
            display_mode,
            top: display_mode.top_interface(),
            slots: HashMap::new(),
        }
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            self.open_top(self.top).await;
            for tab in TabIndex::iter() {
                self.open_slot(InterfaceSlot::Tab(tab), tab.default_interface()).await;
            }
            self.open_subs(OrbIndex::iter().map(OrbIndex::sub)).await;
            self.open_subs(ChatIndex::iter().map(ChatIndex::sub)).await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}
}
