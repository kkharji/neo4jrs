use crate::field_info::{Default, FieldInformation, FieldModifier};
use crate::util;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::str::FromStr;
use syn::DeriveInput;

/// Inject pub presist function to insert struct to graph db
fn presist_fn(name: &Ident, fields: &Vec<FieldInformation>) -> TokenStream {
    let field_names = fields
        .iter()
        .map(|s| format!("{}: ${}", s.name, s.name))
        .collect::<Vec<String>>()
        .join(", ");

    let field_injects = fields.iter().map(|info| {
        let name = &info.name;
        let ident = info.ident();
        quote!(p.put(#name.into(), self.#ident.clone().into()))
    });

    let query = format!("create (_:{name} {{{}}})", field_names);

    quote! {
        pub async fn persist(&self, executor: &impl neo4jrs::Execute) -> neo4jrs::Result<()> {
            let mut p = neo4jrs::types::BoltMap::default() #(; #field_injects)*;

            let query = neo4jrs::Query::new_with_params(#query, p);

            executor.run(query).await
        }
    }
}

fn impl_node(name: &Ident, fields: &Vec<FieldInformation>) -> TokenStream {
    let field_inject = fields.iter().map(|info| {
        let mut or_value = None;
        let name = &info.name;
        let ident = info.ident();

        if !info.field_type.contains("Option") {
            for m in info.modifiers.iter() {
                if let FieldModifier::Default(d) = m {
                    if let Default::Fn(f) = d {
                        or_value = TokenStream::from_str(&format!("{f}()")).ok();
                    } else {
                        or_value = TokenStream::from_str("std::default::Default::default()").ok();
                    }
                } else {
                    continue;
                }
            }
        }

        if info.field_type.contains("Option") {
            quote! { #ident: n.get(#name), }
        } else {
            if let Some(or_value) = or_value {
                quote! { #ident: n.get(#name).unwrap_or(#or_value), }
            } else {
                quote! { #ident: n.get(#name).unwrap(), }
            }
        }
    });

    let expanded = quote! {
        impl From<neo4jrs::Node> for #name {
            fn from(n: neo4jrs::Node) -> Self {
                Self {
                    #(#field_inject)*
                }
            }
        }

    };

    expanded.into()
}

pub fn expand(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = util::collect_fields_information(&ast);
    let presist_fn = presist_fn(name, &fields);
    let impl_node = impl_node(name, &fields);
    let expanded = quote! {
        #impl_node

        impl #name {
            #presist_fn
        }
    };

    // println!("{}", expanded);
    expanded.into()
}
