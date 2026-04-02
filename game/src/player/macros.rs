#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $crate::player::send_message($player, &format!($($arg)*))
    };
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
