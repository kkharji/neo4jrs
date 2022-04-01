use super::{lit, Ctx};
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::str::FromStr;
use syn::{Data, Fields, Lit};

/// Represents the default to use for a field.
#[derive(Debug)]
pub enum Default {
    /// No defaults
    None,
    /// The default is given by `std::default::Default::default()`.
    Default(TokenStream),
    /// Normal string to be put as is
    Custom(TokenStream),
}

impl Default {
    pub fn custom<A: ToTokens>(token: A, cx: &Ctx, lit: &Lit) -> Self {
        if let Some(s) = lit::to_string(lit) {
            match TokenStream::from_str(&format!("{s}()")) {
                Ok(token) => Self::Custom(token),
                Err(e) => {
                    let msg = format!("Fail to convert string to token a string got {:?}", e);
                    cx.error_spanned_by(token, msg);
                    Self::None
                }
            }
        } else {
            let msg = format!("Expected a string got {:?}", lit);
            cx.error_spanned_by(token, msg);
            Self::None
        }
    }

    pub fn default() -> Self {
        TokenStream::from_str("std::default::Default::default()")
            .map(|t| Self::Default(t))
            .unwrap_or(Self::None)
    }

    pub fn from_container_path(input: &syn::DeriveInput, cx: &Ctx) -> Self {
        let err_msg = "#[neo4j(default = \"...\")] can only be used on structs with named fields";
        match &input.data {
            Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                Fields::Named(_) => Self::default(),
                Fields::Unnamed(_) | Fields::Unit => {
                    cx.error_spanned_by(fields, err_msg);
                    Self::None
                }
            },
            Data::Enum(syn::DataEnum { enum_token, .. }) => {
                cx.error_spanned_by(enum_token, err_msg);
                Self::None
            }
            Data::Union(syn::DataUnion { union_token, .. }) => {
                cx.error_spanned_by(union_token, err_msg);
                Self::None
            }
        }
    }

    pub(crate) fn from_container_name_value(
        input: &syn::DeriveInput,
        cx: &Ctx,
        m: &syn::MetaNameValue,
    ) -> Self {
        let err_msg = "#[neo4j(default = \"...\")] can only be used on structs with named fields";
        match &input.data {
            Data::Struct(syn::DataStruct { fields, .. }) => match fields {
                Fields::Named(_) => Default::custom(fields, cx, &m.lit),
                Fields::Unnamed(_) | Fields::Unit => {
                    cx.error_spanned_by(fields, err_msg);
                    Default::None
                }
            },
            Data::Enum(syn::DataEnum { enum_token, .. }) => {
                cx.error_spanned_by(enum_token, err_msg);
                Default::None
            }
            Data::Union(syn::DataUnion { union_token, .. }) => {
                cx.error_spanned_by(union_token, err_msg);
                Default::None
            }
        }
    }
}
