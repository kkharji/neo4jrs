use crate::ast::{Container, Ctx, Derive};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
mod convert;
mod find;
mod insert;
mod persist;
mod update;

pub fn expand(ast: DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctx = Ctx::new();

    let cont = match Container::from_ast(&ctx, &ast, Derive::Label) {
        Some(cont) => cont,
        None => return Err(ctx.check().unwrap_err()),
    };

    let persist = persist::gen(&ctx, &cont);
    let find = find::gen(&ctx, &cont);
    let extend = convert::gen(&ctx, &cont);
    let update = update::gen(&ctx, &cont);
    let insert = insert::gen(&ctx, &cont);

    ctx.check()?;

    let expanded = quote! {
        #insert
        #update
        #persist
        #find
        #extend
    };

    Ok(expanded.into())
}
