#[macro_export]
macro_rules! send_message {
    ($player:expr, $($arg:tt)*) => {
        $crate::player::send_message($player, &format!($($arg)*))
    };
}
