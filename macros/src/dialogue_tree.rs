use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

struct DialogueTree {
    entry: Vec<DialogueItem>,
    sections: Vec<Section>,
}

enum DialogueItem {
    Stmt(TokenStream2),
    Choice(Vec<ChoiceArm>),
    Goto(Ident),
}

struct ChoiceArm {
    text: LitStr,
    target: Ident,
}

struct Section {
    name: Ident,
    items: Vec<DialogueItem>,
}

fn peek_keyword(input: ParseStream, name: &str) -> bool {
    input
        .cursor()
        .ident()
        .is_some_and(|(id, rest)| id == name && rest.punct().is_some_and(|(p, _)| p.as_char() == '!'))
}

fn parse_goto_target(input: ParseStream) -> syn::Result<Ident> {
    input.parse::<Ident>()?;
    input.parse::<Token![!]>()?;
    let content;
    syn::parenthesized!(content in input);
    content.parse()
}

fn parse_item(input: ParseStream) -> syn::Result<DialogueItem> {
    if peek_keyword(input, "dialogue_choice") {
        input.parse::<Ident>()?;
        input.parse::<Token![!]>()?;
        let content;
        syn::braced!(content in input);

        let mut arms = Vec::new();
        while !content.is_empty() {
            let text: LitStr = content.parse()?;
            content.parse::<Token![=>]>()?;
            let target = parse_goto_target(&content)?;
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
            arms.push(ChoiceArm { text, target });
        }
        Ok(DialogueItem::Choice(arms))
    } else if peek_keyword(input, "goto") {
        let target = parse_goto_target(input)?;
        input.parse::<Token![;]>()?;
        Ok(DialogueItem::Goto(target))
    } else {
        let stmt: syn::Stmt = input.parse()?;
        Ok(DialogueItem::Stmt(quote! { #stmt }))
    }
}

impl Parse for DialogueTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut entry = Vec::new();
        let mut sections = Vec::new();

        while !input.is_empty() {
            if peek_keyword(input, "dialogue") {
                input.parse::<Ident>()?;
                input.parse::<Token![!]>()?;
                let name_content;
                syn::parenthesized!(name_content in input);
                let name: Ident = name_content.parse()?;

                let body_content;
                syn::braced!(body_content in input);
                let mut items = Vec::new();
                while !body_content.is_empty() {
                    items.push(parse_item(&body_content)?);
                }
                sections.push(Section { name, items });
            } else {
                entry.push(parse_item(input)?);
            }
        }

        Ok(DialogueTree { entry, sections })
    }
}

fn resolve_target(label: &Ident) -> TokenStream2 {
    if label == "root" {
        quote! { __DialogueState::__entry }
    } else if label == "done" {
        quote! { __DialogueState::__done }
    } else {
        quote! { __DialogueState::#label }
    }
}

fn generate_items(items: &[DialogueItem]) -> TokenStream2 {
    let mut stmts: Vec<TokenStream2> = Vec::new();
    let mut ends_with_transition = false;

    for item in items {
        match item {
            DialogueItem::Stmt(ts) => {
                stmts.push(ts.clone());
                ends_with_transition = false;
            }
            DialogueItem::Choice(arms) => {
                let texts: Vec<_> = arms.iter().map(|a| &a.text).collect();
                let match_arms: Vec<_> = arms
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let target = resolve_target(&a.target);
                        if i == arms.len() - 1 {
                            quote! { _ => #target }
                        } else {
                            let idx = (i + 1) as u8;
                            quote! { #idx => #target }
                        }
                    })
                    .collect();

                stmts.push(quote! {
                    __state = match options_dialogue!(#(#texts),*) {
                        #(#match_arms,)*
                    };
                    continue '__dialogue;
                });
                ends_with_transition = true;
            }
            DialogueItem::Goto(label) => {
                let target = resolve_target(label);
                stmts.push(quote! {
                    __state = #target;
                    continue '__dialogue;
                });
                ends_with_transition = true;
            }
        }
    }

    if !ends_with_transition {
        stmts.push(quote! {
            __state = __DialogueState::__done;
            continue '__dialogue;
        });
    }

    quote! { #(#stmts)* }
}

pub fn dialogue_tree(input: TokenStream) -> TokenStream {
    let tree = syn::parse_macro_input!(input as DialogueTree);

    let section_names: Vec<&Ident> = tree.sections.iter().map(|s| &s.name).collect();
    let entry_body = generate_items(&tree.entry);

    let section_arms: Vec<_> = tree
        .sections
        .iter()
        .map(|s| {
            let name = &s.name;
            let body = generate_items(&s.items);
            quote! { __DialogueState::#name => { #body } }
        })
        .collect();

    quote! {
        {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            enum __DialogueState {
                __entry,
                #(#section_names,)*
                __done,
            }

            macro_rules! goto {
                (root) => {{ __state = __DialogueState::__entry; continue '__dialogue; }};
                (done) => {{ break '__dialogue; }};
                ($label:ident) => {{ __state = __DialogueState::$label; continue '__dialogue; }};
            }

            let mut __state = __DialogueState::__entry;
            '__dialogue: loop {
                match __state {
                    __DialogueState::__entry => { #entry_body }
                    #(#section_arms)*
                    __DialogueState::__done => break,
                }
            }
        }
    }
    .into()
}
