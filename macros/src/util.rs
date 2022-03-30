use crate::field_info::{FieldInformation, FieldModifier};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::ToTokens;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::Data::Struct;
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::{self, Meta};
use syn::{parse_quote, DataStruct, MetaList, MetaNameValue};

pub fn lit_to_string(lit: &syn::Lit) -> Option<String> {
    match *lit {
        Str(ref s) => Some(s.value()),
        _ => None,
    }
}

/// Serde can be used to rename fields on deserialization but most of the times
/// we want the error on the original field.
///
/// For example a JS frontend might send camelCase fields and Rust converts them to snake_case
/// but we want to send the errors back with the original name
/// CREDIT: @ELD
fn find_original_field_name(meta_items: &[&NestedMeta]) -> Option<String> {
    let mut original_name = None;

    for meta_item in meta_items {
        match **meta_item {
            Meta(ref item) => match *item {
                Path(_) => continue,
                NameValue(MetaNameValue {
                    ref path, ref lit, ..
                }) => {
                    let ident = path.get_ident().unwrap();
                    if ident == "rename" {
                        original_name = Some(lit_to_string(lit).unwrap());
                    }
                }
                List(MetaList { ref nested, .. }) => {
                    return find_original_field_name(&nested.iter().collect::<Vec<_>>());
                }
            },
            _ => unreachable!(),
        };

        if original_name.is_some() {
            return original_name;
        }
    }

    original_name
}

/// Find the types (as string) for each field of the struct
/// Needed for the `must_match` filter
fn find_fields_type(fields: &[syn::Field]) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field.ident.clone().unwrap().to_string();
        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference {
                ref lifetime,
                ref elem,
                ..
            }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            _ => {
                let mut field_type = proc_macro2::TokenStream::new();
                field.ty.to_tokens(&mut field_type);
                abort!(
                    field.ty.span(),
                    "Type `{}` of field `{}` not supported",
                    field_type,
                    field_ident
                )
            }
        };

        types.insert(field_ident, field_type);
    }

    types
}

/// Find everything we need to know about a field
fn find_field_modifiers(
    field: &syn::Field,
    field_types: &HashMap<String, String>,
) -> (String, Vec<FieldModifier>) {
    let mut has_modifiers = false;
    let mut modifiers = vec![];
    let mut field_ident = field.ident.clone().unwrap().to_string();

    let _rust_ident = field.ident.clone().unwrap().to_string();
    let _field_type = field_types.get(&field_ident).unwrap();

    let error = |span: Span, msg: &str| -> ! {
        let name = field.ident.clone().unwrap().to_string();
        abort!(
            span,
            "Invalid attribute #[neo4j] on field `{}`: {}",
            name,
            msg
        );
    };

    for attr in &field.attrs {
        if attr.path != parse_quote!(neo4j) && attr.path != parse_quote!(serde) {
            continue;
        }

        if attr.path == parse_quote!(neo4j) {
            has_modifiers = true;
        }

        match attr.parse_meta() {
            Ok(List(MetaList { ref nested, .. })) => {
                let meta_items = nested.iter().collect::<Vec<_>>();
                // original name before serde rename
                if attr.path == parse_quote!(serde) {
                    if let Some(s) = find_original_field_name(&meta_items) {
                        field_ident = s;
                    }
                    continue;
                }
                for meta_item in meta_items {
                    match *meta_item {
                        NestedMeta::Meta(ref item) => match *item {
                            Path(ref name) => {
                                match name.get_ident().unwrap().to_string().as_ref() {
                                    "ignore" => {
                                        modifiers.push(FieldModifier::Ignore);
                                    }
                                    v => abort!(name.span(), "Unexpected field modifier: {}", v),
                                }
                            }
                            NameValue(MetaNameValue {
                                ref path, ref lit, ..
                            }) => {
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "default" => {
                                        match lit_to_string(lit) {
                                            Some(s) => modifiers.push(FieldModifier::Default(s)),
                                            None => error(lit.span(), "invalid argument for `contains` validator: only strings are allowed"),
                                        };
                                    }
                                    v => abort!(
                                        path.span(),
                                        "unexpected name value validator: {:?}",
                                        v
                                    ),
                                };
                            }
                            List(MetaList {
                                ref path,
                                ref nested,
                                ..
                            }) => {
                                let _meta_items = nested.iter().cloned().collect::<Vec<_>>();
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() as &str {
                                    v => abort!(path.span(), "unexpected list modifiers: {:?}", v),
                                }
                            }
                        },
                        _ => unreachable!("Found a non Meta while looking for validators"),
                    };
                }
            }
            Ok(Path(_)) => modifiers.push(FieldModifier::Nested),
            Ok(NameValue(_)) => abort!(attr.span(), "Unexpected name=value argument"),
            Err(e) => {
                let error_string = format!("{:?}", e);
                if error_string == "Error(\"expected literal\")" {
                    abort!(attr.span(),
                        "This attributes for the field `{}` seem to be misformed, please validate the syntax with the documentation",
                        field_ident
                    );
                } else {
                    abort!(
                        attr.span(),
                        "Unable to parse this attribute for the field `{}` with the error: {:?}",
                        field_ident,
                        e
                    );
                }
            }
        }

        if has_modifiers && modifiers.is_empty() {
            error(attr.span(), "it needs at least one validator");
        }
    }

    (field_ident, modifiers)
}

fn collect_fields(ast: &syn::DeriveInput) -> Vec<syn::Field> {
    match ast.data {
        Struct(DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "struct has unnamed fields";
                    help = "Only be used on structs with named fields";
                );
            }
            fields.iter().cloned().collect::<Vec<_>>()
        }
        _ => abort!(ast.span(), "Only be used with structs"),
    }
}

pub fn collect_fields_information(ast: &syn::DeriveInput) -> Vec<FieldInformation> {
    let mut fields = collect_fields(ast);
    let field_types = find_fields_type(&fields);
    fields.drain(..).fold(vec![], |mut acc, field| {
        let key = field.ident.clone().unwrap().to_string();
        let (name, modifiers) = find_field_modifiers(&field, &field_types);
        if modifiers.contains(&FieldModifier::Ignore) {
            return acc;
        }

        acc.push(FieldInformation {
            field,
            field_type: field_types.get(&key).unwrap().clone(),
            name,
            modifiers,
        });
        acc
    })
}
