use quote::quote;

pub fn macros() -> proc_macro2::TokenStream {
    quote! {
        macro_rules! remove_item {
            ($amount:expr) => {
                crate::player::active_player()
                    .inventory_mut()
                    .remove_item(__slot as usize, $amount)
                    .await
            };
        }
        macro_rules! slot_item {
            () => { crate::player::active_player().inventory().slot(__slot as usize) };
        }
        macro_rules! clear_slot {
            () => { crate::player::active_player().inventory_mut().clear_slot(__slot as usize).await };
        }
        macro_rules! drop_to_ground {
            ($item_id:expr, $amount:expr) => {{
                let __player = crate::player::active_player();
                let __pos = __player.position;
                let __world = __player.world();
                __player.ground_item_mut().drop($item_id, $amount, __pos, &__world);
            }};
        }
        macro_rules! item_def {
            ($id:expr) => { crate::provider::get_item_definition($id as u32) };
        }
        macro_rules! equipped {
            ($slot:expr) => { crate::player::active_player().equipment().slot($slot) };
        }
        macro_rules! unequip {
            ($slot:expr) => { crate::player::active_player().equipment_mut().set($slot, None) };
        }
    }
}
