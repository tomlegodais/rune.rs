use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, parse_macro_input};

pub fn message_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let expanded = quote! {
        #func

        inventory::submit! {
            InboundDecoder {
                opcode: OPCODE,
                decode: #func_name,
            }
        }
    };

    expanded.into()
}

pub fn message_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let msg_type = match func.sig.inputs.iter().nth(1) {
        Some(FnArg::Typed(pat_type)) => &*pat_type.ty,
        _ => panic!("message_handler function must have a second parameter for the message type"),
    };

    let wrapper_name = format_ident!("__{}_handler_wrapper", func_name);
    let expanded = quote! {
        #func

        fn #wrapper_name<'a>(
            player: &'a mut crate::player::Player,
            msg: net::IncomingMessage,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async move {
                if let Ok(concrete) = msg.downcast::<#msg_type>() {
                    #func_name(player, *concrete).await;
                }
            })
        }

        inventory::submit! {
            MessageHandler {
                type_id_fn: || std::any::TypeId::of::<#msg_type>(),
                handle: #wrapper_name,
            }
        }
    };

    expanded.into()
}
