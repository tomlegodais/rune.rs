#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $player.send_message(&format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! with_movement {
    ($player:expr, |$m:ident, $ctx:ident| $body:expr) => {{
        let mut $m = $player.systems.guard::<$crate::player::Movement>();
        let mut varps = $player.systems.guard::<$crate::player::VarpManager>();
        let agility_level = $player
            .system::<$crate::player::SkillManager>()
            .level($crate::player::Skill::Agility);

        let mut $ctx = $crate::player::MovementContext {
            position: &mut $player.position,
            player_info: &mut $player.player_info,
            varps: &mut varps,
            agility_level,
            region_base: $player.viewport.region_base,
        };

        $body
    }};
}

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
