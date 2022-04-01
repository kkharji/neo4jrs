// #![allow(dead_code)]

use super::super::{get_neo4j_meta_items, lit, Attr, Ctx, Default, DEFAULT, EXPECTING, IDENTIFER};

use syn::Meta::{NameValue, Path};
use syn::NestedMeta::{Lit, Meta};

/// Represents struct or enum attribute information.
pub struct ContainerAttrs {
    pub name: String,
    /// The main identifier key to search with in database
    pub identifier: String,
    /// Default alternative for missing fields
    pub default: Default,
    /// Error message generated when type can't be processed
    expecting: Option<String>,
}

impl ContainerAttrs {
    /// Extract out the `#[neo4j(...)]` attributes from an item.
    pub(crate) fn from_ast(cx: &Ctx, input: &syn::DeriveInput) -> Self {
        let mut identifier: Attr<String> = Attr::none(cx, IDENTIFER);
        let mut default: Attr<Default> = Attr::none(cx, DEFAULT);
        let expecting: Attr<String> = Attr::none(cx, EXPECTING);
        let items = input
            .attrs
            .iter()
            .flat_map(|attr| get_neo4j_meta_items(cx, attr))
            .flatten();

        for item in items {
            match &item {
                // Parse `#[neo4j(default)]`
                Meta(Path(word)) if word == DEFAULT => {
                    default.set(word, Default::from_container_path(&input, cx))
                }
                // Parse `#[neo4j(default = "...")]`
                Meta(NameValue(m)) if m.path == DEFAULT => {
                    default.set(&m.path, Default::from_container_name_value(&input, cx, m))
                }
                Meta(NameValue(m)) if m.path == IDENTIFER => {
                    identifier.set_opt(&m.path, lit::to_string(&m.lit))
                }
                Lit(lit) => {
                    let msg = "unexpected literal in neo4j container attribute";
                    cx.error_spanned_by(lit, msg);
                }
                x => {
                    let msg = format!("unexpected container attribute in neo4j: {:?}", x);
                    cx.error_spanned_by(item, msg);
                }
            }
        }

        Self {
            name: input.ident.to_string(),
            identifier: identifier.get().unwrap_or("id".into()),
            expecting: expecting.get(),
            default: default.get().unwrap_or(Default::None),
        }
    }

    /// Error message generated when type can't be deserialized.
    /// If `None`, default message will be used
    pub fn expecting(&self) -> Option<&str> {
        self.expecting.as_ref().map(String::as_ref)
    }

    /// Get a reference to the container's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the container's index.
    pub fn identifier(&self) -> &str {
        self.identifier.as_ref()
    }

    /// Get a reference to the container's default.
    pub fn default(&self) -> &Default {
        &self.default
    }
}
