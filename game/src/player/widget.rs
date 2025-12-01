use crate::message::{OpenWidget, SetRootWidget};
use crate::player::{SharedConnection, ui};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    Fixed = 1,
    Resizable = 2,
    FullScreen = 3,
}

pub trait Widget: Send + Sync {
    fn interface(&self) -> u16;
    fn parent(&self, root: RootWidget) -> u16;
    fn position(&self, mode: DisplayMode) -> u16;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RootWidget(pub u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScreenWidget {
    pub interface: u16,
    pub fixed_position: u16,
    pub resizable_position: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChatboxWidget {
    pub interface: u16,
    pub fixed_position: u16,
    pub resizable_position: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InventoryWidget(pub u16);

pub struct WidgetManager {
    connection: SharedConnection,
    display_mode: DisplayMode,
    root: RootWidget,
    widgets: HashMap<u16, u16>,
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

impl RootWidget {
    const FIXED: RootWidget = Self(548);
    const RESIZABLE: RootWidget = Self(746);
}

impl ScreenWidget {
    pub const fn new(interface: u16) -> Self {
        Self::with_position(interface, 6, 9)
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

impl Widget for ScreenWidget {
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
}

impl ChatboxWidget {
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
}

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
}

impl WidgetManager {
    pub fn new(connection: SharedConnection, display_mode: u8) -> Self {
        let display_mode = DisplayMode::from_u8(display_mode).unwrap_or(DisplayMode::Fixed);

        Self {
            connection,
            display_mode,
            root: display_mode.root_widget(),
            widgets: HashMap::new(),
        }
    }

    pub async fn on_login(&mut self) {
        self.set_root_widget(self.root).await;
        self.open_widgets(ui::tabs::widgets()).await;
        self.open_widgets(ui::orbs::widgets()).await;
        self.open_widgets(ui::chat::widgets()).await;
    }

    pub async fn set_root_widget(&mut self, root: RootWidget) {
        if self.root != root {
            self.root = root;
        }

        let mut connection = self.connection.lock().await;
        connection.send(SetRootWidget(root)).await;
    }

    pub async fn open_widget<W>(&mut self, widget: &W)
    where
        W: Widget + ?Sized,
    {
        let position = widget.position(self.display_mode);
        let interface = widget.interface();
        self.widgets.insert(position, interface);

        let mut connection = self.connection.lock().await;
        connection
            .send(OpenWidget {
                parent: widget.parent(self.root),
                position: widget.position(self.display_mode),
                interface,
                click_through: false,
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

    pub fn is_open(&self, widget: impl Widget) -> bool {
        let position = widget.position(self.display_mode);
        self.widgets.get(&position) == Some(&widget.interface())
    }
}
