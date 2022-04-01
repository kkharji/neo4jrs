#![allow(dead_code)]
use super::{Ctx, Field, FieldAttrs};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::Token;

#[derive(Copy, Clone)]
pub enum Style {
    /// Named fields.
    Struct,
    /// Many unnamed fields.
    Tuple,
    /// One unnamed field.
    Newtype,
    /// No fields.
    Unit,
}

/// The fields of a struct or enum.
///
/// Analogous to `syn::Data`.
pub enum Data<'a> {
    // Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

impl<'a> Data<'a> {
    pub fn from_struct(
        cx: &Ctx,
        fields: &'a syn::Fields,
        container_default: &super::Default,
    ) -> (Style, Vec<Field<'a>>) {
        match fields {
            syn::Fields::Named(fields) => (
                Style::Struct,
                Self::fields_from_ast(cx, &fields.named, container_default),
            ),
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => (
                Style::Newtype,
                Self::fields_from_ast(cx, &fields.unnamed, container_default),
            ),
            syn::Fields::Unnamed(fields) => (
                Style::Tuple,
                Self::fields_from_ast(cx, &fields.unnamed, container_default),
            ),
            syn::Fields::Unit => (Style::Unit, Vec::new()),
        }
    }

    fn fields_from_ast(
        cx: &Ctx,
        fields: &'a Punctuated<syn::Field, Token![,]>,
        container_default: &super::Default,
    ) -> Vec<Field<'a>> {
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let mut ty_str = None;
                match &field.ty {
                    syn::Type::Path(syn::TypePath { ref path, .. }) => {
                        let mut tokens = proc_macro2::TokenStream::new();
                        path.to_tokens(&mut tokens);
                        ty_str = Some(tokens.to_string().replace(' ', ""));
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
                        ty_str = Some(name);
                    }
                    syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                        let mut tokens = proc_macro2::TokenStream::new();
                        elem.to_tokens(&mut tokens);
                        ty_str = Some(tokens.to_string().replace(' ', ""));
                    }
                    _ => {
                        let mut field_type = proc_macro2::TokenStream::new();
                        field.ty.to_tokens(&mut field_type);
                        cx.error_spanned_by(
                            &field.ty,
                            format!("Type `{}` is not supported", field_type,),
                        );
                    }
                }

                Field {
                    member: match &field.ident {
                        Some(ident) => syn::Member::Named(ident.clone()),
                        None => syn::Member::Unnamed(i.into()),
                    },
                    attrs: FieldAttrs::from_ast(cx, i, field, container_default),
                    ty: &field.ty,
                    ty_str,
                    original: field,
                }
            })
            .collect()
    }
}
