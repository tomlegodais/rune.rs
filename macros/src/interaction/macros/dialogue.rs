use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! npc_dialogue {
            (id = $id:expr, seq = $seq:expr, $($fmt_args:tt)+) => {{
                let __name = crate::provider::get_npc_type($id as u32)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown");
                let __text = format!($($fmt_args)+);
                crate::player::active_player()
                    .dialogue_mut()
                    .entity_dialogue(crate::player::DialogueEntity::Npc($id as u16, Some($seq as u16)), __name, &__text)
                    .await;
                crate::player::await_dialogue().await;
            }};
            (id = $id:expr, $($fmt_args:tt)+) => {{
                let __name = crate::provider::get_npc_type($id as u32)
                    .map(|t| t.name.as_str())
                    .unwrap_or("Unknown");
                let __text = format!($($fmt_args)+);
                crate::player::active_player()
                    .dialogue_mut()
                    .entity_dialogue(crate::player::DialogueEntity::Npc($id as u16, None), __name, &__text)
                    .await;
                crate::player::await_dialogue().await;
            }};
            (seq = $seq:expr, $($fmt_args:tt)+) => {{
                let __text = format!($($fmt_args)+);
                crate::player::active_player()
                    .dialogue_mut()
                    .entity_dialogue(crate::player::DialogueEntity::Npc(__npc_id, Some($seq as u16)), __npc_name, &__text)
                    .await;
                crate::player::await_dialogue().await;
            }};
            ($($fmt_args:tt)+) => {{
                let __text = format!($($fmt_args)+);
                crate::player::active_player()
                    .dialogue_mut()
                    .entity_dialogue(crate::player::DialogueEntity::Npc(__npc_id, None), __npc_name, &__text)
                    .await;
                crate::player::await_dialogue().await;
            }};
        }

        macro_rules! player_dialogue {
            (seq = $seq:expr, $($fmt_args:tt)+) => {{
                let __text = format!($($fmt_args)+);
                crate::player::active_player()
                    .dialogue_mut()
                    .entity_dialogue(crate::player::DialogueEntity::Player(Some($seq as u16)), &crate::player::active_player().username, &__text)
                    .await;
                crate::player::await_dialogue().await;
            }};
            ($($fmt_args:tt)+) => {{
                let __text = format!($($fmt_args)+);
                crate::player::active_player()
                    .dialogue_mut()
                    .entity_dialogue(crate::player::DialogueEntity::Player(None), &crate::player::active_player().username, &__text)
                    .await;
                crate::player::await_dialogue().await;
            }};
        }

        macro_rules! options_dialogue {
            ($($opt:expr),+ $(,)?) => {{
                crate::player::active_player().dialogue_mut().option_dialogue(&[$($opt),+]).await;
                crate::player::await_dialogue().await + 1 - crate::player::OPTIONS_FIRST_COMPONENT as u8
            }};
        }

        macro_rules! dialogue_tree {
            ($($tt:tt)*) => { macros::dialogue_tree!($($tt)*) };
        }
    }
}
