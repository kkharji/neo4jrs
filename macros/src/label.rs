use crate::field_info::FieldInformation;
use crate::util;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
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

pub fn expand(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = util::collect_fields_information(&ast);
    let presist_fn = presist_fn(name, &fields);

    let expanded = quote! {
        impl #name {
            #presist_fn
        }
    };

    // println!("{}", expanded);
    expanded.into()
}
