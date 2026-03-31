use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! loc_replace {
            (replace = $replace:expr, ticks = $ticks:expr) => {
                loc_replace!(@__impl Some($replace as u16), $ticks)
            };
            (ticks = $ticks:expr) => {
                loc_replace!(@__impl None, $ticks)
            };
            (@__impl $replacement:expr, $ticks:expr) => {{
                let __player = crate::player::active_player();
                let __pos = crate::world::Position::new(loc_x as i32, loc_y as i32, __player.position.plane);
                let __collision = crate::provider::get_collision();
                if let Some(__original) = __collision.get_loc(__pos, loc_id as u32) {
                    __player.world().locs.replace(__pos, __original, $replacement, $ticks);
                }
            }};
        }
        macro_rules! loc_replaced {
            () => {{
                let __player = crate::player::active_player();
                let __pos = crate::world::Position::new(loc_x as i32, loc_y as i32, __player.position.plane);
                __player.world().locs.is_replaced(__pos, loc_id as u32)
            }};
        }
    }
}
