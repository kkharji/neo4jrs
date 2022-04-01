use syn::Type;

#[derive(Copy, Clone)]
pub enum Derive {
    Label,
    #[allow(dead_code)]
    Relation,
}

#[allow(dead_code)]
pub fn ungroup(mut ty: &Type) -> &Type {
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
