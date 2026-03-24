pub mod tabs {
    use crate::player::WindowWidget;

    widget_group! {
        pub const ATTACK: WindowWidget = WindowWidget::new(884, 152, 33);
        pub const SKILLS: WindowWidget = WindowWidget::new(320, 153, 34);
        pub const QUESTS: WindowWidget = WindowWidget::new(190, 154, 35);
        pub const ACHIEVEMENTS: WindowWidget = WindowWidget::new(259, 155, 36);
        pub const INVENTORY: WindowWidget = WindowWidget::new(149, 156, 37);
        pub const EQUIPMENT: WindowWidget = WindowWidget::new(387, 157, 38);
        pub const PRAYER: WindowWidget = WindowWidget::new(271, 158, 39);
        pub const MAGIC: WindowWidget = WindowWidget::new(192, 159, 40);
        pub const OBJECTIVE: WindowWidget = WindowWidget::new(891, 160, 41);
        pub const FRIENDS: WindowWidget = WindowWidget::new(550, 161, 42);
        pub const IGNORES: WindowWidget = WindowWidget::new(551, 162, 43);
        pub const CLAN: WindowWidget = WindowWidget::new(589, 163, 44);
        pub const SETTINGS: WindowWidget = WindowWidget::new(261, 164, 45);
        pub const EMOTES: WindowWidget = WindowWidget::new(464, 165, 46);
        pub const MUSIC: WindowWidget = WindowWidget::new(187, 166, 47);
        pub const NOTES: WindowWidget = WindowWidget::new(34, 167, 48);
        pub const LOGOUT: WindowWidget = WindowWidget::new(182, 170, 51);
    }
}

pub mod orbs {
    use crate::player::WindowWidget;

    widget_group! {
        pub const SUMMONING: WindowWidget = WindowWidget::new(747, 139, 172);
        pub const HITPOINTS: WindowWidget = WindowWidget::new(748, 134, 169);
        pub const PRAYER: WindowWidget = WindowWidget::new(749, 136, 170);
        pub const RUN_ENERGY: WindowWidget = WindowWidget::new(750, 137, 171);
    }
}

pub mod chat {
    use crate::player::{ChatboxWidget, WindowWidget};

    widget_group! {
        pub const CHAT_FRAME: WindowWidget = WindowWidget::new(752, 142, 18);
        pub const CHAT_AREA: ChatboxWidget = ChatboxWidget::with_static_position(137, 9);
        pub const CHAT_OPTIONS: WindowWidget = WindowWidget::new(751, 20, 15);
        pub const PRIVATE_CHAT_AREA: WindowWidget = WindowWidget::new(754, 14, 19);
    }
}
