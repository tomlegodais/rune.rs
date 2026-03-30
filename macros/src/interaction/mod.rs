use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

pub mod interface;
pub mod item;
pub mod npc;
pub mod object;
pub mod player;

pub enum AttrValue {
    Int(syn::LitInt),
    Ident(syn::Ident),
}

impl Parse for AttrValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) { Ok(AttrValue::Int(input.parse()?)) } else { Ok(AttrValue::Ident(input.parse()?)) }
    }
}

pub struct InteractionAttr {
    pub pairs: Vec<(String, AttrValue)>,
}

impl Parse for InteractionAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut pairs = Vec::new();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: AttrValue = input.parse()?;
            pairs.push((ident.to_string(), value));
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(InteractionAttr { pairs })
    }
}

impl InteractionAttr {
    fn get_value<T>(&self, key: &str, f: impl Fn(&AttrValue) -> Option<&T>) -> Option<&T> {
        self.pairs.iter().find_map(|(k, v)| (k == key).then(|| f(v)).flatten())
    }

    pub fn get_int(&self, key: &str) -> Option<&syn::LitInt> {
        self.get_value(key, |v| if let AttrValue::Int(i) = v { Some(i) } else { None })
    }

    pub fn require_int(&self, key: &str) -> syn::Result<&syn::LitInt> {
        self.get_int(key)
            .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), format!("missing `{key}`")))
    }

    pub fn get_ident(&self, key: &str) -> Option<&syn::Ident> {
        self.get_value(key, |v| if let AttrValue::Ident(i) = v { Some(i) } else { None })
    }

    pub fn option_variant(&self) -> syn::Result<proc_macro2::TokenStream> {
        if let Some(ident) = self.get_ident("option") {
            return match ident.to_string().as_str() {
                "One" | "Two" | "Three" | "Four" | "Five" | "Six" | "Seven" | "Eight" | "Nine" | "Ten" => {
                    Ok(quote! { net::ClickOption::#ident })
                }
                _ => Err(syn::Error::new(ident.span(), "option must be One..Ten")),
            };
        }
        let opt = self.require_int("option")?;
        let n: u8 = opt.base10_parse()?;
        let variant = format_ident!(
            "{}",
            match n {
                1 => "One",
                2 => "Two",
                3 => "Three",
                4 => "Four",
                5 => "Five",
                6 => "Six",
                7 => "Seven",
                8 => "Eight",
                9 => "Nine",
                10 => "Ten",
                _ => return Err(syn::Error::new(opt.span(), "option must be 1-10")),
            }
        );
        Ok(quote! { net::ClickOption::#variant })
    }
}

pub fn emit_content_handler(
    wrapper_name: &syn::Ident,
    target_expr: proc_macro2::TokenStream,
    destructure: proc_macro2::TokenStream,
    bindings: proc_macro2::TokenStream,
    macros: proc_macro2::TokenStream,
    func_body: &syn::Block,
) -> TokenStream {
    quote! {
        #[allow(unused_macros, unused_variables)]
        fn #wrapper_name(
            target: crate::player::InteractionTarget,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>> {
            #destructure
            Box::pin(async move {
                let __shared = crate::player::active_shared();
                #bindings
                #macros
                #func_body
            })
        }

        inventory::submit! {
            crate::handler::ContentHandler {
                target: #target_expr,
                handle: #wrapper_name,
            }
        }
    }
    .into()
}

pub fn base_macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! send_message { ($($a:tt)*) => { crate::player::send_message(crate::player::active_player(), &format!($($a)*)) }; }
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
        macro_rules! successful {
            (chance = $chance:expr) => {
                rand::random::<f64>() < $chance
            };
        }
        macro_rules! depleted {
            (chance = $chance:expr) => {
                rand::random::<f64>() < $chance
            };
        }
        macro_rules! requires {
            (skill = $skill:ident, level = $lvl:expr) => {
                if crate::player::active_player().skill().level(crate::player::Skill::$skill) < $lvl {
                    send_message!(
                        "You need a {} level of {} to do that.",
                        stringify!($skill),
                        $lvl
                    );
                    return;
                }
            };
            (skill = $skill:ident, level = $lvl:expr, $msg:expr) => {
                if crate::player::active_player().skill().level(crate::player::Skill::$skill) < $lvl {
                    send_message!($msg);
                    return;
                }
            };
            (inventory, slots = $n:expr) => {
                if crate::player::active_player().inventory().free_slots() < $n {
                    send_message!("Your inventory is too full.");
                    return;
                }
            };
        }
        macro_rules! inv_add {
            (id = $id:expr) => {
                crate::player::active_player().inventory_mut().add($id, 1).await;
            };
            (id = $id:expr, amount = $n:expr) => {
                crate::player::active_player().inventory_mut().add($id, $n).await;
            };
        }
        macro_rules! give_xp {
            (skill = $skill:ident, amount = $xp:expr) => {
                crate::player::active_player().skill_mut().add_xp(crate::player::Skill::$skill, $xp).await;
            };
        }
        macro_rules! inventory_full {
            () => {
                crate::player::active_player().inventory().free_slots() == 0
            };
        }
        macro_rules! anim {
            ($id:expr) => { crate::player::active_player().anim($id) };
            ($id:expr, $($k:ident = $v:expr),+) => { { let b = crate::player::active_player().anim($id); $(let b = b.$k($v);)+ b } };
        }
        macro_rules! spotanim {
            ($id:expr) => { crate::player::active_player().spot_anim($id) };
            ($id:expr, $($k:ident = $v:expr),+) => { { let b = crate::player::active_player().spot_anim($id); $(let b = b.$k($v);)+ b } };
        }
    }
}
