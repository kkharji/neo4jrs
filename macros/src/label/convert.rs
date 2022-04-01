//! implement conversion types
use crate::ast::{iter_fields, Container, Ctx, Default};
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen(cx: &Ctx, cont: &Container) -> TokenStream {
    let name = cont.ident();
    let injections = iter_fields(cx, cont, |field| {
        let fname = field.attrs.name();
        let ident = field.original.ident.as_ref()?;
        let is_optional = field.is_optional();
        Some(match field.attrs.default() {
            Default::None => {
                if is_optional {
                    quote!(#ident: n.get(#fname),)
                } else {
                    quote!(#ident: n.get(#fname).unwrap(),)
                }
            }
            Default::Default(or_value) | Default::Custom(or_value) => {
                if is_optional {
                    quote!(#ident: Some(n.get(#fname).unwrap_or(#or_value)),)
                } else {
                    quote!(#ident: n.get(#fname).unwrap_or(#or_value),)
                }
            }
        })
    });

    // TODO: convert to bolt type instead
    let expanded = quote! {
        impl From<neo4jrs::Node> for #name {
            fn from(n: neo4jrs::Node) -> Self { Self { #(#injections)* } }
        }
    };

    expanded.into()
}
