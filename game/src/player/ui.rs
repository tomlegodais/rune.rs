macro_rules! widget_group {
    (
        $(
            $vis:vis const $name:ident : $ty:ty = $expr:expr;
        )+
    ) => {
        $(
            $vis const $name: $ty = $expr;
        )+

        pub fn widgets() -> impl Iterator<Item = &'static dyn $crate::player::widget::Widget> {
            [
                $(
                    &$name as &dyn $crate::player::Widget,
                )+
            ]
            .into_iter()
        }
    };
}

pub mod tabs {
    use crate::player::ScreenWidget;

    widget_group! {
        pub const ATTACK: ScreenWidget = ScreenWidget::with_position(884, 152, 33);
        pub const SKILLS: ScreenWidget = ScreenWidget::with_position(320, 153, 34);
        pub const QUESTS: ScreenWidget = ScreenWidget::with_position(190, 154, 35);
        pub const ACHIEVEMENTS: ScreenWidget = ScreenWidget::with_position(259, 155, 36);
        pub const INVENTORY: ScreenWidget = ScreenWidget::with_position(149, 156, 37);
        pub const EQUIPMENT: ScreenWidget = ScreenWidget::with_position(387, 157, 38);
        pub const PRAYER: ScreenWidget = ScreenWidget::with_position(271, 158, 39);
        pub const MAGIC: ScreenWidget = ScreenWidget::with_position(192, 159, 40);
        pub const OBJECTIVE: ScreenWidget = ScreenWidget::with_position(891, 160, 41);
        pub const FRIENDS: ScreenWidget = ScreenWidget::with_position(550, 161, 42);
        pub const IGNORES: ScreenWidget = ScreenWidget::with_position(551, 162, 43);
        pub const CLAN: ScreenWidget = ScreenWidget::with_position(589, 163, 44);
        pub const SETTINGS: ScreenWidget = ScreenWidget::with_position(261, 164, 45);
        pub const EMOTES: ScreenWidget = ScreenWidget::with_position(464, 165, 46);
        pub const MUSIC: ScreenWidget = ScreenWidget::with_position(187, 166, 47);
        pub const NOTES: ScreenWidget = ScreenWidget::with_position(34, 167, 48);
        pub const LOGOUT: ScreenWidget = ScreenWidget::with_position(182, 170, 51);
    }
}

pub mod orbs {
    use crate::player::{ScreenWidget, Widget};

    widget_group! {
        pub const SUMMONING: ScreenWidget = ScreenWidget::with_position(747, 139, 172);
        pub const HITPOINTS: ScreenWidget = ScreenWidget::with_position(748, 134, 169);
        pub const PRAYER: ScreenWidget = ScreenWidget::with_position(749, 136, 170);
        pub const RUN_ENERGY: ScreenWidget = ScreenWidget::with_position(750, 137, 172);
    }
}

pub mod chat {
    use crate::player::{ChatboxWidget, ScreenWidget, Widget};

    widget_group! {
        pub const CHAT_FRAME: ScreenWidget = ScreenWidget::with_position(752, 142, 18);
        pub const CHAT_AREA: ChatboxWidget = ChatboxWidget::with_static_position(137, 9);
        pub const CHAT_OPTIONS: ScreenWidget = ScreenWidget::with_position(751, 20, 15);
        pub const PRIVATE_CHAT_AREA: ScreenWidget = ScreenWidget::with_position(754, 14, 19);
    }
}
