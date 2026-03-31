use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

pub mod interface;
pub mod item;
pub mod npc;
pub mod loc;
pub mod player;
pub mod macros;

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

