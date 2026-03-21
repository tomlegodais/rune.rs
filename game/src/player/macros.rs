#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $player.send_message(&format!($($arg)*)).await
    };
}

#[macro_export]
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
