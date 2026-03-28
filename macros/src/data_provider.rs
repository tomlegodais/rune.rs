use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{ItemFn, LitInt, Token, parse_macro_input};

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
                        "expected `priority`",
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
