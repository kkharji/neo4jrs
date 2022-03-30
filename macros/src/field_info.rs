use proc_macro2::Ident;

/// Modifiers to customize the field.
#[derive(Debug, PartialEq)]
pub enum FieldModifier {
    Ignore,
    Default(String),
    Nested,
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
