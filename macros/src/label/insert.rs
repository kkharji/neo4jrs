//! Inject function to insert  struct to database.
use crate::ast::{iter_fields, Container, Ctx};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn gen(cx: &Ctx, cont: &Container) -> TokenStream {
    let name = cont.ident();

    let (create_fields, injections): (Vec<String>, Vec<TokenStream>) =
        iter_fields(cx, cont, |field| {
            let name = field.attrs.name();
            let ident = field.original.ident.as_ref()?;
            let injection = quote!(p.put(#name.into(), self.#ident.clone().into()));
            let kv = format!("{}: ${}", name, name);

            Some((kv, injection))
        })
        .into_iter()
        .unzip();

    let query = format!("create (_:{} {{{}}})", name, create_fields.join(", "));

    quote! {
        impl #name {
            pub async fn insert(&self, graph: &impl neo4jrs::Execute) -> neo4jrs::Result<()> {
                use neo4jrs::Query;
                let mut p = neo4jrs::types::BoltMap::default() #(; #injections)*;
                let q = neo4jrs::Query::new_with_params(#query, p);
                graph.run(q).await
            }
        }
    }
}
