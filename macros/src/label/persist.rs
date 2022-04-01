//! Inject function to update or insert struct to database.
use crate::ast::{Container, Ctx};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn gen(_cx: &Ctx, cont: &Container) -> TokenStream {
    let identifier = cont.attrs.identifier();
    let identifier_ident = format_ident!("{identifier}");
    let name = cont.ident();
    let find_query = format!(
        "match (n:{name} {{{}: ${}}}) return n.id",
        identifier, identifier
    );

    quote! {
        impl #name {
            pub async fn persist(&self, graph: &impl neo4jrs::Execute) -> neo4jrs::Result<()> {
                let find_query = neo4jrs::Query::new(#find_query).param(#identifier, self.#identifier_ident.clone());

                if let Ok(mut result) = graph.execute(find_query).await {
                    if let Ok(Some(row)) = result.next().await {
                        if row.get::<neo4jrs::types::BoltString>("n.id").is_some() {
                            return self.update(graph).await
                        }
                    }
                }

                tracing::trace!("Inserting new entity");
                self.insert(graph).await
            }
        }
    }
}
