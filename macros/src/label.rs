use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Data::Struct;
use syn::{DeriveInput, Error, Field, Fields};

/// Inject pub presist function to insert struct to graph db
fn presist(name: &Ident, fields: &Vec<(&Ident, String)>) -> TokenStream {
    let fields_str = fields
        .iter()
        .map(|s| format!("{}: ${}", s.1, s.1))
        .collect::<Vec<String>>()
        .join(", ");

    let fileds_p = fields
        .iter()
        .map(|(id, k)| quote!(p.put(#k.into(), self.#id.clone().into())));

    let query = format!("create (_:{name} {{{}}})", fields_str);

    quote! {
        pub async fn persist(&self, executor: &impl neo4jrs::Execute) -> neo4jrs::Result<()> {
            let mut p = neo4jrs::types::BoltMap::default() #(; #fileds_p)*;

            let query = neo4jrs::Query::new_with_params(#query, p);

            executor.run(query).await
        }
    }
}

fn get_fields<'a>(struct_fields: &'a Punctuated<Field, Comma>) -> Vec<(&'a Ident, String)> {
    struct_fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref()?;
            let string = ident.to_string();
            Some((ident, string))
        })
        .flatten()
        .collect::<Vec<(&syn::Ident, String)>>()
}

fn get_struct_fields(ast: &DeriveInput) -> Result<Punctuated<Field, Comma>, Vec<Error>> {
    if let Struct(data) = &ast.data {
        match &data.fields {
            Fields::Named(syn::FieldsNamed { named, .. }) => Ok(named.clone()),
            Fields::Unnamed(_) => Err(vec![Error::new(
                ast.ident.span(),
                "unnamed fields not supported",
            )]),
            Fields::Unit => Ok(Punctuated::new()),
        }
    } else {
        return Err(vec![Error::new(ast.ident.span(), "not a struct")]);
    }
}

pub fn expand(ast: DeriveInput) -> Result<TokenStream, Vec<Error>> {
    let name = &ast.ident;
    let struct_fields = match get_struct_fields(&ast) {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    let fields = get_fields(&struct_fields);
    let presist = presist(name, &fields);

    let gen = quote! {
        impl #name {
            #presist
        }
    };

    println!("{}", gen);

    Ok(gen.into())
}
