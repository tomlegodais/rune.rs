#[macro_export]
macro_rules! movement_ctx {
    ($self:ident) => {
        &mut $crate::player::MovementContext {
            position: &mut $self.position,
            player_info: &mut $self.player_info,
            varps: &mut $self.varps,
            agility_level: $self.skills.level($crate::player::Skill::Agility),
            region_base: $self.viewport.region_base,
        }
    };
}

#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $player.send_message(&format!($($arg)*)).await
    };
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
