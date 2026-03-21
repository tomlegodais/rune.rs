use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, GenericArgument, ItemFn, LitStr, Pat, PathArguments, Token, Type, parse_macro_input,
};

#[proc_macro_attribute]
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

#[proc_macro_attribute]
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

struct CommandAttr {
    name: LitStr,
}

impl syn::parse::Parse for CommandAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "name" {
            return Err(syn::Error::new(ident.span(), "expected `name`"));
        }
        input.parse::<Token![=]>()?;
        let name: LitStr = input.parse()?;
        Ok(CommandAttr { name })
    }
}

fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last()?;
        if segment.ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(GenericArgument::Type(inner)) = args.args.first() {
                    return Some(inner);
                }
            }
        }
    }
    None
}

fn extract_param_name(pat: &Pat) -> syn::Ident {
    match pat {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        _ => panic!("command parameters must be simple identifiers"),
    }
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as CommandAttr);
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let cmd_name = &attr.name;
    let wrapper_name = format_ident!("__{}_command_wrapper", func_name);
    let params: Vec<_> = func
        .sig
        .inputs
        .iter()
        .skip(1)
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some((extract_param_name(&pat_type.pat), &*pat_type.ty))
            } else {
                None
            }
        })
        .collect();

    let usage_parts: Vec<String> = params
        .iter()
        .map(|(name, ty)| {
            if extract_option_inner(ty).is_some() {
                format!("[{}]", name)
            } else {
                format!("<lt>{}<gt>", name)
            }
        })
        .collect();

    let usage_lit = LitStr::new(
        &format!("Usage: ::{} {}", cmd_name.value(), usage_parts.join(" ")),
        cmd_name.span(),
    );
    let mut parse_stmts = Vec::new();
    let mut param_names = Vec::new();

    for (i, (name, ty)) in params.iter().enumerate() {
        param_names.push(name.clone());

        if let Some(inner_ty) = extract_option_inner(ty) {
            parse_stmts.push(quote! {
                let #name: Option<#inner_ty> = __args.get(#i)
                    .and_then(|s| s.parse().ok());
            });
        } else {
            parse_stmts.push(quote! {
                let #name: #ty = match __args.get(#i).and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None => {
                        crate::send_message!(player, "{}", #usage_lit);
                        return;
                    }
                };
            });
        }
    }

    let expanded = quote! {
        #func

        fn #wrapper_name<'a>(
            player: &'a mut crate::player::Player,
            __raw_args: &'a str,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async move {
                let __args: Vec<&str> = if __raw_args.is_empty() {
                    Vec::new()
                } else {
                    __raw_args.split_whitespace().collect()
                };

                #(#parse_stmts)*

                #func_name(player, #(#param_names),*).await;
            })
        }

        inventory::submit! {
            CommandEntry {
                name: #cmd_name,
                handle: #wrapper_name,
            }
        }
    };

    expanded.into()
}
