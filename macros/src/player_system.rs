use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemImpl, Type, parse_macro_input};

fn system_accessor_name(ty: &Type) -> syn::Ident {
    let name = match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default(),
        _ => panic!("player_system: unsupported type"),
    };

    let stripped = ["Manager", "System", "Tracker"]
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(&name);

    let snake = stripped
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                vec!['_', c.to_lowercase().next().unwrap()]
            } else {
                vec![c.to_lowercase().next().unwrap()]
            }
        })
        .collect::<String>();

    format_ident!("{}", snake)
}

pub fn player_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    let self_ty = &impl_block.self_ty;
    let accessor = system_accessor_name(self_ty);
    let accessor_mut = format_ident!("{}_mut", accessor);

    let expanded = quote! {
        #impl_block

        impl crate::player::Player {
            pub fn #accessor(&self) -> &'_ #self_ty {
                self.systems.get::<#self_ty>()
            }

            pub fn #accessor_mut(&mut self) -> &'_ mut #self_ty {
                self.systems.get_mut::<#self_ty>()
            }
        }

        inventory::submit! {
            crate::player::system::SystemRegistration {
                type_id: || std::any::TypeId::of::<#self_ty>(),
                deps: <#self_ty as crate::player::system::PlayerSystem>::dependencies,
                factory: |ctx| Box::new(<#self_ty as crate::player::system::PlayerSystem>::create(ctx)),
                persist: |any, data| {
                    any.downcast_ref::<#self_ty>().unwrap().persist(data);
                },
                on_login: |any, ctx| {
                    any.downcast_mut::<#self_ty>().unwrap().on_login(ctx)
                },
                tick_context: |world, player| Box::new(<#self_ty as crate::player::system::PlayerSystem>::tick_context(world, player)),
                tick: |any, ctx| {
                    any.downcast_mut::<#self_ty>().unwrap().tick(
                        ctx.downcast_ref::<<#self_ty as crate::player::system::PlayerSystem>::TickContext>().unwrap()
                    )
                },
            }
        }
    };

    expanded.into()
}
