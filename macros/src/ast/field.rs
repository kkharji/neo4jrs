mod attrs;
pub use attrs::FieldAttrs;

/// A field of a struct.
pub struct Field<'a> {
    pub member: syn::Member,
    pub attrs: FieldAttrs,
    pub ty: &'a syn::Type,
    pub ty_str: Option<String>,
    pub original: &'a syn::Field,
}

impl<'a> Field<'a> {
    pub fn is_optional(&self) -> bool {
        self.ty_str
            .as_ref()
            .unwrap_or(&"".to_string())
            .contains("Option")
    }
}
