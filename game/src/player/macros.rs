#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $crate::player::send_message($player, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! delay {
    ($ticks:expr) => {
        $crate::player::delay(&__shared, $ticks)
    };
}

#[macro_export]
macro_rules! npc_force_talk {
    ($($arg:tt)*) => {
        $crate::player::npc_force_talk(__player, __npc_index, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lock {
    () => {
        $crate::player::lock(&__shared)
    };
}

#[macro_export]
macro_rules! unlock {
    () => {
        $crate::player::unlock(&__shared)
    };
}

#[macro_export]
macro_rules! skill_action {
    () => {
        $crate::player::SkillActionBuilder::new(__shared.clone())
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
