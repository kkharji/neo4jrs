use crate::ast::{iter_fields, Container, Ctx};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn gen(cx: &Ctx, cont: &Container) -> TokenStream {
    let name = cont.ident();
    let get_all_query = format!("match (n:{name}) return n");
    let find_fns = iter_fields(cx, cont, |field| {
        let fname = field.attrs.name();
        let by_many_ident = format_ident!("find_many_by_{fname}");
        let by_one_ident = format_ident!("find_one_by_{fname}");
        let query = format!("match (n:{name} {{{fname}: ${fname}}}) return n");

        Some(quote! {
            pub async fn #by_many_ident<T: Into<neo4jrs::types::BoltType>>(val: T, graph: &impl neo4jrs::Execute) -> Result<Vec<Self>, neo4jrs::Error> {
                let query = neo4jrs::Query::new(#query).param(#fname, val);
                Self::query(query, graph).await
            }

            pub async fn #by_one_ident<T: Into<neo4jrs::types::BoltType>>(val: T, graph: &impl neo4jrs::Execute) -> Result<Self, neo4jrs::Error> {
                let query = neo4jrs::Query::new(#query).param(#fname, val);
                Self::query_one(query, graph).await
            }
        })
    });

    let expanded = quote! {

        impl #name {

            pub async fn get_all(graph: &impl neo4jrs::Execute) -> Result<Vec<Self>, neo4jrs::Error> {
                println!(#get_all_query);
                Self::query(neo4jrs::Query::new(#get_all_query), graph).await
            }

            pub async fn query(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Result<Vec<Self>, neo4jrs::Error> {
                let mut list = vec![];
                let mut result = graph.execute(query).await?;

                while let Ok(Some(row)) = result.next().await {
                    if let Some(n) = row.get::<neo4jrs::Node>("n") {
                        list.push(n.into());
                    }
                }

                Ok(list)
            }

            pub async fn query_one(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Result<Self, neo4jrs::Error> {
                if let Ok(Some(row)) = graph.execute(query).await?.next().await {
                    if let Some(n) = row.get::<neo4jrs::Node>("n") {
                        return Ok(n.into())
                    }
                }

                Err(neo4jrs::Error::NoMatch)
            }

            #(#find_fns)*
        }
    };

    expanded.into()
}
