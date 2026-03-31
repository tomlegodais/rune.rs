#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $crate::player::send_message($player, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! with_movement {
    ($player:expr, |$m:ident, $ctx:ident| $body:expr) => {{
        let mut $m = $player.systems.guard::<$crate::player::Movement>();
        let mut varps = $player.systems.guard::<$crate::player::VarpManager>();
        let agility_level = $player.stat().level($crate::player::Stat::Agility);

        let mut $ctx = $crate::player::MovementContext {
            entity: &mut $player.entity,
            player_info: &mut $player.player_info,
            varps: &mut varps,
            agility_level,
            region_base: $player.viewport.region_base,
        };

        $body
    }};
}

macro_rules! interface_group {
    (
        $(
            $vis:vis const $name:ident : SubInterface = $expr:expr;
        )+
    ) => {
        $(
            $vis const $name: $crate::player::SubInterface = $expr;
        )+

        pub fn interfaces() -> impl Iterator<Item = &'static $crate::player::SubInterface> {
            [
                $(
                    &$name,
                )+
            ]
            .into_iter()
        }
    };
}
