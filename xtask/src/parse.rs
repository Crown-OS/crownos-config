//! Reading `section!` declarations and the public types around them out of the schema sources.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Expr, Fields, Ident, Item, ItemEnum, ItemStruct, LitStr, Token, Type, Visibility,
};

use crate::model::{Field, Section, StructField, TypeDecl, TypeKind, Variant};
use crate::util::{docs, types};

/// Everything found under one schema directory.
#[derive(Debug, Default)]
pub(crate) struct Schema {
    pub(crate) sections: Vec<Section>,
    pub(crate) types: Vec<TypeDecl>,
}

pub(crate) fn read_schema(dir: &Path) -> Result<Schema> {
    let mut schema = Schema::default();
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    files.sort();
    for path in files {
        let source = fs::read_to_string(&path).with_context(|| path.display().to_string())?;
        let file = syn::parse_file(&source).with_context(|| path.display().to_string())?;
        let module_docs = docs::lines(&file.attrs);
        for item in file.items {
            match item {
                Item::Macro(invocation) if is_section_macro(&invocation.mac.path) => {
                    let declaration: SectionDecl = syn::parse2(invocation.mac.tokens)
                        .with_context(|| format!("section! in {}", path.display()))?;
                    schema
                        .sections
                        .push(declaration.into_section(module_docs.clone()));
                }
                Item::Enum(item) if is_public(&item.vis) => schema.types.push(enum_decl(&item)),
                Item::Struct(item) if is_public(&item.vis) => schema.types.push(struct_decl(&item)),
                _ => {}
            }
        }
    }
    schema.sections.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(schema)
}

fn collect_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn is_section_macro(path: &syn::Path) -> bool {
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "section")
}

const fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// The body of a `section!` invocation, as `key.rs` defines it.
struct SectionDecl {
    attrs: Vec<Attribute>,
    name: Ident,
    file: LitStr,
    fields: Vec<FieldDecl>,
}

struct FieldDecl {
    attrs: Vec<Attribute>,
    name: Ident,
    ty: Type,
    default: Expr,
}

impl Parse for SectionDecl {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Visibility>()?;
        input.parse::<Token![struct]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let file: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let keys_keyword: Ident = input.parse()?;
        if keys_keyword != "keys" {
            return Err(syn::Error::new(keys_keyword.span(), "expected `keys`"));
        }
        input.parse::<Ident>()?;
        let body;
        syn::braced!(body in input);
        let mut fields = Vec::new();
        while !body.is_empty() {
            fields.push(body.parse()?);
            if body.peek(Token![,]) {
                body.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            attrs,
            name,
            file,
            fields,
        })
    }
}

impl Parse for FieldDecl {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Visibility>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![as]>()?;
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let default: Expr = input.parse()?;
        Ok(Self {
            attrs,
            name,
            ty,
            default,
        })
    }
}

impl SectionDecl {
    fn into_section(self, module_docs: Vec<String>) -> Section {
        Section {
            struct_name: self.name.to_string(),
            file: self.file.value(),
            docs: docs::lines(&self.attrs),
            module_docs,
            fields: self
                .fields
                .into_iter()
                .map(|field| Field {
                    name: field.name.to_string(),
                    ty: types::text(&field.ty),
                    type_names: types::names(&field.ty),
                    default_source: field.default.to_token_stream().to_string(),
                    docs: docs::lines(&field.attrs),
                })
                .collect(),
        }
    }
}

fn enum_decl(item: &ItemEnum) -> TypeDecl {
    TypeDecl {
        name: item.ident.to_string(),
        docs: docs::lines(&item.attrs),
        kind: TypeKind::Enum(
            item.variants
                .iter()
                .map(|variant| Variant {
                    name: variant.ident.to_string(),
                    payload: payload(&variant.fields),
                    is_default: docs::marks_default(&variant.attrs),
                    docs: docs::lines(&variant.attrs),
                })
                .collect(),
        ),
    }
}

fn payload(fields: &Fields) -> String {
    match fields {
        Fields::Unit => String::new(),
        Fields::Unnamed(unnamed) => format!(
            "({})",
            unnamed
                .unnamed
                .iter()
                .map(|field| types::text(&field.ty))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Fields::Named(named) => format!(
            "{{ {} }}",
            named
                .named
                .iter()
                .map(|field| {
                    let name = field
                        .ident
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default();
                    format!("{name}: {}", types::text(&field.ty))
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn struct_decl(item: &ItemStruct) -> TypeDecl {
    let fields = match &item.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| StructField {
                name: field
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                ty: types::text(&field.ty),
                type_names: types::names(&field.ty),
                docs: docs::lines(&field.attrs),
            })
            .collect(),
        _ => Vec::new(),
    };
    TypeDecl {
        name: item.ident.to_string(),
        docs: docs::lines(&item.attrs),
        kind: TypeKind::Struct(fields),
    }
}
