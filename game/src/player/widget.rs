use crate::player::system::{PlayerInitContext, PlayerSystem, SystemContext};
use crate::player::ui;
use macros::player_system;
use net::{OpenWidget, Outbox, OutboxExt, SetRootWidget};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    Fixed = 1,
    Resizable = 2,
    FullScreen = 3,
}

impl DisplayMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(DisplayMode::Fixed),
            2 => Some(DisplayMode::Resizable),
            3 => Some(DisplayMode::FullScreen),
            _ => None,
        }
    }

    pub fn root_widget(&self) -> RootWidget {
        match self {
            DisplayMode::Fixed => RootWidget::FIXED,
            DisplayMode::Resizable | DisplayMode::FullScreen => RootWidget::RESIZABLE,
        }
    }
}

pub trait Widget: Send + Sync {
    fn interface(&self) -> u16;
    fn parent(&self, root: RootWidget) -> u16;
    fn position(&self, mode: DisplayMode) -> u16;
    fn transparent(&self) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RootWidget(pub u16);

impl RootWidget {
    const FIXED: RootWidget = Self(548);
    const RESIZABLE: RootWidget = Self(746);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ScreenWidget(pub u16);

impl Widget for ScreenWidget {
    fn interface(&self) -> u16 {
        self.0
    }

    fn parent(&self, root: RootWidget) -> u16 {
        root.0
    }

    fn position(&self, mode: DisplayMode) -> u16 {
        match mode {
            DisplayMode::Fixed => 6,
            DisplayMode::Resizable | DisplayMode::FullScreen => 8,
        }
    }

    fn transparent(&self) -> bool {
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WindowWidget {
    pub interface: u16,
    pub fixed_position: u16,
    pub resizable_position: u16,
    pub transparent: bool,
}

impl WindowWidget {
    pub const fn new(interface: u16, fixed_position: u16, resizable_position: u16) -> Self {
        Self {
            interface,
            fixed_position,
            resizable_position,
            transparent: true,
        }
    }

    #[allow(dead_code)]
    pub const fn opaque(mut self) -> Self {
        self.transparent = false;
        self
    }
}

impl Widget for WindowWidget {
    fn interface(&self) -> u16 {
        self.interface
    }

    fn parent(&self, root: RootWidget) -> u16 {
        root.0
    }

    fn position(&self, mode: DisplayMode) -> u16 {
        match mode {
            DisplayMode::Fixed => self.fixed_position,
            DisplayMode::Resizable | DisplayMode::FullScreen => self.resizable_position,
        }
    }

    fn transparent(&self) -> bool {
        self.transparent
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct OverlayWidget(u16);

impl Widget for OverlayWidget {
    fn interface(&self) -> u16 {
        self.0
    }

    fn parent(&self, root: RootWidget) -> u16 {
        root.0
    }

    fn position(&self, mode: DisplayMode) -> u16 {
        match mode {
            DisplayMode::Fixed => 1,
            DisplayMode::Resizable | DisplayMode::FullScreen => 0,
        }
    }

    fn transparent(&self) -> bool {
        true
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChatboxWidget {
    pub interface: u16,
    pub fixed_position: u16,
    pub resizable_position: u16,
}

impl ChatboxWidget {
    #[allow(dead_code)]
    pub const fn new(interface: u16) -> Self {
        Self::with_static_position(interface, 13)
    }

    pub const fn with_static_position(interface: u16, position: u16) -> Self {
        Self::with_position(interface, position, position)
    }

    pub const fn with_position(
        interface: u16,
        fixed_position: u16,
        resizable_position: u16,
    ) -> Self {
        Self {
            interface,
            fixed_position,
            resizable_position,
        }
    }
}

impl Widget for ChatboxWidget {
    fn interface(&self) -> u16 {
        self.interface
    }

    fn parent(&self, _root: RootWidget) -> u16 {
        752
    }

    fn position(&self, mode: DisplayMode) -> u16 {
        match mode {
            DisplayMode::Fixed => self.fixed_position,
            DisplayMode::Resizable | DisplayMode::FullScreen => self.resizable_position,
        }
    }

    fn transparent(&self) -> bool {
        true
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct InventoryWidget(pub u16);

impl Widget for InventoryWidget {
    fn interface(&self) -> u16 {
        self.0
    }

    fn parent(&self, root: RootWidget) -> u16 {
        root.0
    }

    fn position(&self, mode: DisplayMode) -> u16 {
        match mode {
            DisplayMode::Fixed => 147,
            DisplayMode::Resizable | DisplayMode::FullScreen => 30,
        }
    }

    fn transparent(&self) -> bool {
        false
    }
}

pub struct WidgetManager {
    outbox: Outbox,
    display_mode: DisplayMode,
    root: RootWidget,
    widgets: HashMap<u16, u16>,
}

impl WidgetManager {
    pub fn new(outbox: Outbox, display_mode: u8) -> Self {
        let display_mode = DisplayMode::from_u8(display_mode).unwrap_or(DisplayMode::Fixed);

        Self {
            outbox,
            display_mode,
            root: display_mode.root_widget(),
            widgets: HashMap::new(),
        }
    }

    pub async fn set_root_widget(&mut self, root: RootWidget) {
        if self.root != root {
            self.root = root;
        }

        self.outbox.write(SetRootWidget(root.0)).await;
    }

    pub async fn open_widget<W>(&mut self, widget: &W)
    where
        W: Widget + ?Sized,
    {
        let position = widget.position(self.display_mode);
        let interface = widget.interface();
        self.widgets.insert(position, interface);

        self.outbox
            .write(OpenWidget {
                parent: widget.parent(self.root),
                position,
                interface,
                transparent: widget.transparent(),
            })
            .await;
    }

    pub async fn open_widgets<I>(&mut self, widgets: I)
    where
        I: IntoIterator<Item = &'static dyn Widget>,
    {
        for widget in widgets {
            self.open_widget(widget).await;
        }
    }

    #[allow(dead_code)]
    pub fn is_open(&self, widget: impl Widget) -> bool {
        let position = widget.position(self.display_mode);
        self.widgets.get(&position) == Some(&widget.interface())
    }
}

#[player_system]
impl PlayerSystem for WidgetManager {
    fn create(ctx: &PlayerInitContext) -> Self {
        Self::new(ctx.outbox.clone(), ctx.display_mode)
    }

    fn on_login<'a>(
        &'a mut self,
        _ctx: &'a mut SystemContext<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            self.set_root_widget(self.root).await;
            self.open_widgets(ui::tabs::widgets()).await;
            self.open_widgets(ui::orbs::widgets()).await;
            self.open_widgets(ui::chat::widgets()).await;
        })
    }
}
