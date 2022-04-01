use crate::ast::{iter_fields, Container, Ctx};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn gen(cx: &Ctx, cont: &Container) -> TokenStream {
    let name = cont.ident();
    let find_fns = iter_fields(cx, cont, |field| {
        let fname = field.attrs.name();
        let by_many_ident = format_ident!("find_many_by_{fname}");
        let by_one_ident = format_ident!("find_one_by_{fname}");
        let query = format!("match (n:{name} {{{fname}: ${fname}}}) return n");

        Some(quote! {
            pub async fn #by_many_ident<T: Into<neo4jrs::types::BoltType>>(val: T, graph: &impl neo4jrs::Execute) -> Vec<Self> {
                let query = neo4jrs::Query::new(#query).param(#fname, val.into());
                Self::query(query, graph).await
            }

            pub async fn #by_one_ident<T: Into<neo4jrs::types::BoltType>>(val: T, graph: &impl neo4jrs::Execute) -> Option<Self> {
                let query = neo4jrs::Query::new(#query).param(#fname, val.into());
                Self::query_one(query, graph).await
            }
        })
    });

    let expanded = quote! {

        impl #name {
            pub async fn query(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Option<Vec<Self>> {
            pub async fn query(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Vec<Self> {
                let mut list: Vec<Self> = vec![];
                let mut result = match graph.execute(query).await.ok() {
                    Some(r) => r,
                    None => return list,
                };

                while let Ok(Some(row)) = result.next().await {
                    if let Some(n) = row.get::<neo4jrs::Node>("n") {
                        list.push(n.into());
                    }
                }

                list
            }

            pub async fn query_one(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Option<Self> {
                if let Ok(Some(row)) = graph.execute(query).await.ok()?.next().await {
                    if let Some(n) = row.get::<neo4jrs::Node>("n") {
                        return Some(n.into())
                    }
                }

                None
            }

            #(#find_fns)*
        }
    };

    expanded.into()
}
