use super::super::{get_neo4j_meta_items, Attr, BoolAttr, Ctx, Default, DEFAULT, SKIP};
use syn::Meta::{NameValue, Path};
use syn::NestedMeta::{Lit, Meta};

/// Represents field attribute information
pub struct FieldAttrs {
    name: String,
    skip: bool,
    default: Default,
}

impl FieldAttrs {
    pub fn from_ast(
        cx: &Ctx,
        index: usize,
        field: &syn::Field,
        _container_default: &Default,
    ) -> Self {
        let mut skip = BoolAttr::none(cx, SKIP);
        let mut default = Attr::none(cx, DEFAULT);

        let name = match &field.ident {
            Some(ident) => ident.to_string(),
            None => index.to_string(),
        };

        let items = field
            .attrs
            .iter()
            .flat_map(|attr| get_neo4j_meta_items(cx, attr))
            .flatten();

        for item in items {
            match &item {
                // Parse `#[neo4j(skip)]`
                Meta(Path(word)) if word == SKIP => {
                    skip.set_true(word);
                }

                // Parse `#[neo4j(default)]`
                Meta(Path(word)) if word == DEFAULT => {
                    default.set(word, Default::default());
                }

                // Parse `#[neo4j(default = "...")]`
                Meta(NameValue(m)) if m.path == DEFAULT => {
                    default.set(&m.path, Default::custom(m, cx, &m.lit))
                }

                Lit(lit) => {
                    cx.error_spanned_by(lit, "unexpected literal in neo4j container attribute");
                }
                x => {
                    cx.error_spanned_by(
                        &item,
                        format!("unexpected container attribute in neo4j: {:?}", x),
                    );
                }
            }
        }

        Self {
            name,
            skip: skip.get(),
            default: default.get().unwrap_or(Default::None),
        }
    }

    /// Get the field's skip.
    pub fn skip(&self) -> bool {
        self.skip
    }

    /// Get a reference to the field's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the field's default.
    pub fn default(&self) -> &Default {
        &self.default
    }
}
