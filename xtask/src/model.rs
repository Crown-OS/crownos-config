//! What the schema sources say, stripped to what the reference shows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Section {
    pub(crate) struct_name: String,
    pub(crate) file: String,
    pub(crate) docs: Vec<String>,
    pub(crate) module_docs: Vec<String>,
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) ty: String,
    /// Every type name the field's type mentions, for linking to [`TypeDecl`]s.
    pub(crate) type_names: Vec<String>,
    pub(crate) default_source: String,
    pub(crate) docs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeDecl {
    pub(crate) name: String,
    pub(crate) docs: Vec<String>,
    pub(crate) kind: TypeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeKind {
    Enum(Vec<Variant>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Variant {
    pub(crate) name: String,
    /// `(String)` for a tuple variant, `{ a: u8 }` for a struct variant, empty otherwise.
    pub(crate) payload: String,
    pub(crate) is_default: bool,
    pub(crate) docs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructField {
    pub(crate) name: String,
    pub(crate) ty: String,
    pub(crate) type_names: Vec<String>,
    pub(crate) docs: Vec<String>,
}

impl TypeDecl {
    pub(crate) fn mentioned_types(&self) -> impl Iterator<Item = &str> {
        match &self.kind {
            TypeKind::Enum(_) => Vec::new().into_iter(),
            TypeKind::Struct(fields) => fields
                .iter()
                .flat_map(|field| field.type_names.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}
