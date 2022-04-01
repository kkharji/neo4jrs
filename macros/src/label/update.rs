use crate::ast::{iter_fields, Container, Ctx};
use proc_macro2::TokenStream;
use quote::quote;

/// Inject pub update function to insert struct to graph db
pub fn gen(cx: &Ctx, cont: &Container) -> TokenStream {
    let identifier = cont.attrs.identifier();
    let name = cont.ident();

    let (fields_kv, injections): (Vec<String>, Vec<TokenStream>) = iter_fields(cx, cont, |field| {
        let field_name = field.attrs.name();
        let fident = field.original.ident.as_ref()?;
        let injection = quote!(p.put(#field_name.into(), self.#fident.clone().into()));
        let kv = format!("set n.{} = ${}", field_name, field_name);

        Some((kv, injection))
    })
    .into_iter()
    .unzip();

    let update_query = format!(
        "match (n:{name} {{{}: ${}}}) {} return n",
        identifier,
        identifier,
        fields_kv.join("  ")
    );

    let expanded = quote! {
        impl #name {
            pub async fn update(&self, graph: &impl neo4jrs::Execute) -> Result<(), neo4jrs::Error> {
                let mut p = neo4jrs::types::BoltMap::default() #(; #injections)*;
                let query = neo4jrs::Query::new_with_params(#update_query, p);
                graph.run(query).await
            }
        }
    };

    expanded.into()
}
