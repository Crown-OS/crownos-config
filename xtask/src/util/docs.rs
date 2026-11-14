//! Doc comments as the lines a reader sees.

use syn::{Attribute, Expr, Lit, Meta};

/// The `///` and `//!` lines of `attrs`, one string per line, with the leading space removed.
pub(crate) fn lines(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(pair) if pair.path.is_ident("doc") => match &pair.value {
                Expr::Lit(literal) => match &literal.lit {
                    Lit::Str(text) => Some(text.value()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .map(|line| line.strip_prefix(' ').unwrap_or(&line).to_owned())
        .collect()
}

/// Whether `attrs` carry `#[default]`.
pub(crate) fn marks_default(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("default"))
}

/// Doc lines joined into one paragraph for a table cell.
pub(crate) fn one_line(docs: &[String]) -> String {
    let joined = docs
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    joined.replace('|', "\\|")
}

/// The first paragraph of a doc comment.
pub(crate) fn first_paragraph(docs: &[String]) -> String {
    let paragraph: Vec<&str> = docs
        .iter()
        .map(|line| line.trim())
        .take_while(|line| !line.is_empty())
        .collect();
    paragraph.join(" ")
}
