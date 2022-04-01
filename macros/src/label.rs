use crate::ast::{iter_fields, Container, Ctx, Default, Derive};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

/// Inject pub presist function to insert struct to graph db
fn presist_fn(cx: &Ctx, cont: &Container) -> TokenStream {
    let (fields_kv, injections): (Vec<String>, Vec<TokenStream>) = iter_fields(cx, cont, |field| {
        let name = field.attrs.name();
        let ident = field.original.ident.as_ref()?;
        let injection = quote!(p.put(#name.into(), self.#ident.clone().into()));
        let kv = format!("{}: ${}", name, name);

        Some((kv, injection))
    })
    .into_iter()
    .unzip();

    let name = cont.ident();
    let create_query = format!("create (_:{name} {{{}}})", fields_kv.join(", "));

    quote! {
        impl #name {
            pub async fn persist(&self, graph: &impl neo4jrs::Execute) -> neo4jrs::Result<()> {
                let mut p = neo4jrs::types::BoltMap::default() #(; #injections)*;
                let query = neo4jrs::Query::new_with_params(#create_query, p);
                graph.run(query).await
            }
        }
    }
}

/// Inject pub update function to insert struct to graph db
fn update_fn(cx: &Ctx, cont: &Container) -> Option<TokenStream> {
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

    Some(expanded.into())
}

fn impl_node(cx: &Ctx, cont: &Container) -> TokenStream {
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

fn find_by_fn(cx: &Ctx, cont: &Container) -> TokenStream {
    let name = cont.ident();
    let find_fns = iter_fields(cx, cont, |field| {
        let fname = field.attrs.name();
        let by_many_ident = format_ident!("find_many_by_{fname}");
        let by_one_ident = format_ident!("find_one_by_{fname}");
        let query = format!("match (n:{name} {{{fname}: ${fname}}}) return n");

        Some(quote! {
            pub async fn #by_many_ident<T: Into<neo4jrs::types::BoltType>>(val: T, graph: &impl neo4jrs::Execute) -> Option<Vec<Self>> {
                let query = neo4jrs::Query::new(#query).param(#fname, val.into());
                Self::find_many(query, graph).await
            }

            pub async fn #by_one_ident<T: Into<neo4jrs::types::BoltType>>(val: T, graph: &impl neo4jrs::Execute) -> Option<Self> {
                let query = neo4jrs::Query::new(#query).param(#fname, val.into());
                Self::find_one(query, graph).await
            }
        })
    });

    let expanded = quote! {

        impl #name {
            pub async fn find_many(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Option<Vec<Self>> {
                let mut list: Vec<Self> = vec![];
                let mut result = graph.execute(query).await.ok()?;

                while let Ok(Some(row)) = result.next().await {
                    if let Some(n) = row.get::<neo4jrs::Node>("n") {
                        list.push(n.into());
                    }
                }

                Some(list)
            }

            pub async fn find_one(query: neo4jrs::Query, graph: &impl neo4jrs::Execute) -> Option<Self> {
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

pub fn expand(ast: DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctx = Ctx::new();

    let cont = match Container::from_ast(&ctx, &ast, Derive::Label) {
        Some(cont) => cont,
        None => return Err(ctx.check().unwrap_err()),
    };

    let presist = presist_fn(&ctx, &cont);
    let find_by = find_by_fn(&ctx, &cont);
    let impl_node = impl_node(&ctx, &cont);
    let update = update_fn(&ctx, &cont).unwrap_or(quote! {});

    ctx.check()?;

    let expanded = quote! {
        #presist
        #update
        #find_by
        #impl_node
    };

    Ok(expanded.into())
}
