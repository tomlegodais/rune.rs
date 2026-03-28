use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, ItemFn, LitInt, Pat, Token};

pub mod item;
pub mod npc;
pub mod object;
pub mod player;

pub struct InteractionAttr {
    pub pairs: Vec<(String, LitInt)>,
}

impl Parse for InteractionAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut pairs = Vec::new();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitInt = input.parse()?;
            pairs.push((ident.to_string(), value));
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(InteractionAttr { pairs })
    }
}

impl InteractionAttr {
    pub fn get(&self, key: &str) -> Option<&LitInt> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn require(&self, key: &str) -> syn::Result<&LitInt> {
        self.get(key).ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), format!("missing `{key}`"))
        })
    }

    pub fn option_variant(&self) -> syn::Result<proc_macro2::TokenStream> {
        let opt = self.require("option")?;
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
                _ => return Err(syn::Error::new(opt.span(), "option must be 1-8")),
            }
        );
        Ok(quote! { net::ClickOption::#variant })
    }
}

pub fn extract_param_name(pat: &Pat) -> syn::Ident {
    match pat {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        _ => panic!("command parameters must be simple identifiers"),
    }
}

pub fn extract_params(func: &ItemFn) -> Vec<syn::Ident> {
    func.sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(extract_param_name(&pat_type.pat))
            } else {
                None
            }
        })
        .collect()
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
        macro_rules! delay { ($t:expr) => { crate::player::delay(&__shared, $t).await }; }
        macro_rules! lock { () => { crate::player::lock(&__shared) }; }
        macro_rules! unlock { () => { crate::player::unlock(&__shared) }; }
        macro_rules! skill_action { () => { crate::player::SkillActionBuilder::new(__shared.clone()) }; }
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
