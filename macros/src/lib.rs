use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    FnArg, GenericArgument, ItemFn, ItemImpl, LitInt, LitStr, Pat, PathArguments, Token, Type,
    parse_macro_input,
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

impl Parse for CommandAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident != "name" {
            return Err(syn::Error::new(ident.span(), "expected `name`"));
        }
        input.parse::<Token![=]>()?;
        let name: LitStr = input.parse()?;
        Ok(CommandAttr { name })
    }
}

fn is_raw_args_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "RawArgs")
    } else {
        false
    }
}

fn extract_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last()?;
        if segment.ident == "Option"
            && let PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(GenericArgument::Type(inner)) = args.args.first()
        {
            return Some(inner);
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

    let all_params: Vec<_> = func
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

    let has_client_sent = all_params
        .first()
        .map(|(name, _)| name == "client_sent")
        .unwrap_or(false);

    let params = if has_client_sent {
        &all_params[1..]
    } else {
        &all_params[..]
    };

    let usage_parts: Vec<String> = params
        .iter()
        .filter(|(_, ty)| !is_raw_args_type(ty))
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

        if is_raw_args_type(ty) {
            parse_stmts.push(quote! {
                let #name = crate::command::RawArgs(__raw_args.to_string());
            });
        } else if let Some(inner_ty) = extract_option_inner(ty) {
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

    let client_sent_arg = if has_client_sent {
        quote! { __client_sent, }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #func

        fn #wrapper_name<'a>(
            player: &'a mut crate::player::Player,
            __client_sent: bool,
            __raw_args: &'a str,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
            Box::pin(async move {
                let __args: Vec<&str> = if __raw_args.is_empty() {
                    Vec::new()
                } else {
                    __raw_args.split_whitespace().collect()
                };

                #(#parse_stmts)*

                #func_name(player, #client_sent_arg #(#param_names),*).await;
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

#[proc_macro_attribute]
pub fn player_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    let self_ty = &impl_block.self_ty;

    let expanded = quote! {
        #impl_block

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
            }
        }
    };

    expanded.into()
}

struct Args {
    priority: Option<u8>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut priority = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "priority" => {
                    let lit: LitInt = input.parse()?;
                    priority = Some(lit.base10_parse::<u8>()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "expected `priority` or `name`",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Args { priority })
    }
}

#[proc_macro_attribute]
pub fn data_provider(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let func = parse_macro_input!(item as ItemFn);
    let name = &func.sig.ident;
    let name_str = name.to_string();
    let priority = args.priority.unwrap_or(0);
    let submit_ident = syn::Ident::new(
        &format!("__REGISTER_PROVIDER_{}", name_str.to_uppercase()),
        name.span(),
    );

    quote! {
        #func

        inventory::submit! {
            crate::provider::DataProvider {
                priority: #priority,
                load: #name,
            }
        }

        #[allow(dead_code)]
        const #submit_ident: () = ();
    }
    .into()
}
