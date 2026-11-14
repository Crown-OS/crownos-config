//! Types as text, and the names they mention.

use quote::ToTokens;
use syn::{GenericArgument, PathArguments, Type};

/// `Vec<WindowRule>` rather than `Vec < WindowRule >`.
pub(crate) fn text(ty: &Type) -> String {
    let raw = ty.to_token_stream().to_string();
    raw.replace(" < ", "<")
        .replace(" >", ">")
        .replace("< ", "<")
        .replace(" ,", ",")
        .replace(" :: ", "::")
        .replace("[ ", "[")
        .replace(" ]", "]")
        .replace(" ;", ";")
}

/// Every path type name in `ty`, outermost first: `Vec<Option<Foo>>` yields `Vec`, `Option`,
/// `Foo`.
pub(crate) fn names(ty: &Type) -> Vec<String> {
    let mut found = Vec::new();
    collect(ty, &mut found);
    found
}

fn collect(ty: &Type, found: &mut Vec<String>) {
    match ty {
        Type::Path(path) => {
            for segment in &path.path.segments {
                found.push(segment.ident.to_string());
                if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    for argument in &arguments.args {
                        if let GenericArgument::Type(inner) = argument {
                            collect(inner, found);
                        }
                    }
                }
            }
        }
        Type::Array(array) => collect(&array.elem, found),
        Type::Slice(slice) => collect(&slice.elem, found),
        Type::Tuple(tuple) => tuple.elems.iter().for_each(|elem| collect(elem, found)),
        Type::Reference(reference) => collect(&reference.elem, found),
        Type::Paren(paren) => collect(&paren.elem, found),
        Type::Group(group) => collect(&group.elem, found),
        _ => {}
    }
}
