#![allow(dead_code)]
mod attrs;

use super::{Ctx, Data, Derive};
pub use attrs::ContainerAttrs;

/// A source data structure annotated with `#[derive(Serialize)]` and/or `#[derive(Deserialize)]`,
/// parsed into an internal representation.
pub struct Container<'a> {
    /// The struct or enum name (without generics).
    ident: syn::Ident,
    /// Attributes on the structure
    pub attrs: ContainerAttrs,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
    /// Original input.
    pub original: &'a syn::DeriveInput,
}

impl<'a> Container<'a> {
    pub fn from_ast(
        cx: &Ctx,
        item: &'a syn::DeriveInput,
        _derive: Derive,
    ) -> Option<Container<'a>> {
        let attrs = ContainerAttrs::from_ast(cx, item);

        let data = match &item.data {
            syn::Data::Enum(_) => {
                cx.error_spanned_by(item, "Enums no supported");
                return None;
            }
            syn::Data::Struct(data) => {
                let (style, fields) = Data::from_struct(cx, &data.fields, attrs.default());
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                cx.error_spanned_by(item, "no support derive for unions");
                return None;
            }
        };

        let item = Container {
            ident: item.ident.clone(),
            attrs,
            data,
            generics: &item.generics,
            original: item,
        };

        Some(item)
    }

    /// Get a reference to the container's ident.
    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }
}
