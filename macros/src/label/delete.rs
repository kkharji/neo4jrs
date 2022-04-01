use crate::ast::{Container, Ctx};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Inject pub update function to insert struct to graph db
pub fn gen(_cx: &Ctx, cont: &Container) -> TokenStream {
    let identifier = cont.attrs.identifier();
    let identifier_ident = format_ident!("{}", identifier);
    let name = cont.ident();

    let delete_query = format!(
        "match (n:{name} {{{}: ${}}}) delete n",
        identifier, identifier,
    );

    let expanded = quote! {
        impl #name {
            pub async fn delete(&self, graph: &impl neo4jrs::Execute) -> Result<(), neo4jrs::Error> {
                let query = neo4jrs::Query::new(#delete_query).param(#identifier, self.#identifier_ident.clone());
                graph.run(query).await
            }
        }
    };

    expanded.into()
}
