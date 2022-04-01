mod attr;
mod container;
mod ctx;
mod data;
mod default;
mod derive;
mod field;
pub mod lit;
mod symbol;

pub use attr::*;
pub use container::*;
pub use ctx::Ctx;
pub use data::*;
pub use default::Default;
pub use derive::{ungroup, Derive};
pub use field::*;
pub use symbol::*;
use syn::Meta::List;

pub fn iter_fields<T, F>(cx: &Ctx, cont: &Container, transform: F) -> Vec<T>
where
    F: Fn(&Field) -> Option<T>,
{
    match &cont.data {
        Data::Struct(Style::Struct, fields) => fields
            .iter()
            .map(|field| {
                if field.attrs.skip() {
                    None
                } else {
                    transform(field)
                }
            })
            .flatten()
            .collect(),
        _ => {
            cx.error_spanned_by(cont.ident(), "Only struct supported");
            vec![]
        }
    }
}

pub fn get_neo4j_meta_items(cx: &Ctx, attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, ()> {
    if attr.path != NEO4J {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(other) => {
            cx.error_spanned_by(other, "expected #[neo4j(...)]");
            Err(())
        }
        Err(err) => {
            cx.syn_error(err);
            Err(())
        }
    }
}
