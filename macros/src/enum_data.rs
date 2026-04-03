use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Ident, ItemEnum, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct EnumDataAttr {
    with_discriminant: bool,
    types: Punctuated<Type, Token![,]>,
}

impl Parse for EnumDataAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let with_discriminant = if input.peek(syn::Ident) && input.peek2(Token![,]) {
            let ident: syn::Ident = input.fork().parse()?;
            if ident == "discriminant" {
                input.parse::<syn::Ident>()?;
                input.parse::<Token![,]>()?;
                true
            } else {
                false
            }
        } else {
            false
        };
        Ok(Self {
            with_discriminant,
            types: Punctuated::parse_terminated(input)?,
        })
    }
}

struct VariantData {
    ident: Ident,
    values: Punctuated<Expr, Token![,]>,
}

pub fn enum_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as EnumDataAttr);
    let mut input = parse_macro_input!(item as ItemEnum);
    let types: Vec<&Type> = attr.types.iter().collect();
    let n = types.len();
    let mut variants_data: Vec<VariantData> = Vec::new();

    for variant in &mut input.variants {
        let ident = variant.ident.clone();
        let discriminant = variant
            .discriminant
            .take()
            .expect("every variant must have an `= (...)` discriminant");

        let Expr::Tuple(mut tuple) = discriminant.1 else {
            panic!("variant `{}` discriminant must be a tuple `= (...)`", ident);
        };

        let disc = if attr.with_discriminant {
            assert_eq!(
                tuple.elems.len(),
                n + 1,
                "variant `{}` has {} elements but expected {} (discriminant + {} data fields)",
                ident,
                tuple.elems.len(),
                n + 1,
                n
            );

            let disc = tuple.elems.first().cloned();
            let rest: Punctuated<Expr, Token![,]> = tuple.elems.into_iter().skip(1).collect();
            tuple.elems = rest;
            disc
        } else {
            assert_eq!(
                tuple.elems.len(),
                n,
                "variant `{}` has {} elements but attribute specifies {}",
                ident,
                tuple.elems.len(),
                n
            );
            None
        };

        if let Some(ref d) = disc {
            variant.discriminant = Some((syn::token::Eq::default(), d.clone()));
        }

        variants_data.push(VariantData {
            ident,
            values: tuple.elems,
        });
    }

    let enum_name = &input.ident;
    let match_arms: Vec<TokenStream2> = variants_data
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let values = &v.values;
            quote! { Self::#ident => (#values), }
        })
        .collect();

    let ret_type = if n == 1 {
        let t = types[0];
        quote! { #t }
    } else {
        quote! { (#(#types),*) }
    };

    let expanded = quote! {
        #input

        impl #enum_name {
            pub const fn data(self) -> #ret_type {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    expanded.into()
}
