use proc_macro2::Ident;

/// Modifiers to customize the field.
#[derive(Debug, PartialEq)]
pub enum FieldModifier {
    Ignore,
    Default(Default),
    Nested,
}

/// Represents the default to use for a field.
#[derive(Debug, PartialEq)]
pub enum Default {
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// Normal string to be put as is
    Fn(String),
}

/// Field information for one filed
#[derive(Debug)]
pub struct FieldInformation {
    pub field: syn::Field,
    pub field_type: String,
    pub name: String,
    pub modifiers: Vec<FieldModifier>,
}

impl FieldInformation {
    pub fn ident(&self) -> &Ident {
        self.field.ident.as_ref().unwrap()
    }
}
