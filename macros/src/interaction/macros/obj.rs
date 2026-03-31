use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! remove_obj {
            ($amount:expr) => {
                crate::player::active_player()
                    .inv_mut()
                    .remove_obj(__slot as usize, $amount)
                    .await
            };
        }
        macro_rules! slot_obj {
            () => { crate::player::active_player().inv().slot(__slot as usize) };
        }
        macro_rules! clear_slot {
            () => { crate::player::active_player().inv_mut().clear_slot(__slot as usize).await };
        }
        macro_rules! drop_to_ground {
            ($obj_id:expr, $amount:expr) => {{
                let __player = crate::player::active_player();
                let __pos = __player.position;
                let __world = __player.world();
                __player.obj_stack_mut().drop($obj_id, $amount, __pos, &__world);
            }};
        }
        macro_rules! obj_def {
            ($id:expr) => { crate::provider::get_obj_type($id as u32) };
        }
        macro_rules! worn {
            ($slot:expr) => { crate::player::active_player().worn().slot($slot) };
        }
        macro_rules! unwear {
            ($slot:expr) => { crate::player::active_player().worn_mut().set($slot, None) };
        }
    }
}
