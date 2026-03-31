use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! delay { ($t:expr) => { crate::player::delay(&__shared, $t).await }; }
        macro_rules! lock { () => { crate::player::lock(&__shared) }; }
        macro_rules! unlock { () => { crate::player::unlock(&__shared) }; }
        macro_rules! repeat {
            (delay = $d:expr, anim = $a:expr, times = $t:expr, $body:block) => {
                repeat!(@__impl $d, $t, Some($a), $body)
            };
            (delay = $d:expr, anim = $a:expr, $body:block) => {
                repeat!(@__impl $d, 0, Some($a), $body)
            };
            (delay = $d:expr, times = $t:expr, $body:block) => {
                repeat!(@__impl $d, $t, None::<u16>, $body)
            };
            (delay = $d:expr, $body:block) => {
                repeat!(@__impl $d, 0, None::<u16>, $body)
            };
            (@__impl $d:expr, $t:expr, $a:expr, $body:block) => {{
                let __max_iters: u32 = $t;
                let mut __iter_count: u32 = 0;
                let __anim_id: Option<u16> = $a;
                let __anim_guard = crate::player::AnimResetGuard(crate::player::active_player() as *mut _);
                loop {
                    if let Some(id) = __anim_id {
                        crate::player::active_player().anim(id);
                    }
                    crate::player::delay(&__shared, $d).await;
                    __iter_count += 1;
                    $body
                    if __max_iters > 0 && __iter_count >= __max_iters {
                        break;
                    }
                }
                drop(__anim_guard);
            }};
        }
        macro_rules! with_movement {
            ($player:expr, |$m:ident, $ctx:ident| $body:expr) => {{
                let mut $m = $player.systems.guard::<crate::player::Movement>();
                let mut varps = $player.systems.guard::<crate::player::VarpManager>();
                let agility_level = $player.skill().level(crate::player::Skill::Agility);
                let mut $ctx = crate::player::MovementContext {
                    entity: &mut $player.entity,
                    player_info: &mut $player.player_info,
                    varps: &mut varps,
                    agility_level,
                    region_base: $player.viewport.region_base,
                };
                $body
            }};
        }
    }
}
