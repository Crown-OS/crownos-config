//! Markdown for one section and for the index.

use std::collections::BTreeSet;

use crownos_config::DynSection;

use crate::model::{Section, TypeDecl, TypeKind};
use crate::util::docs;

const CONFIG_DIR: &str = "~/.config/crownos";
/// A default longer than this goes under the table instead of inside a cell.
const INLINE_DEFAULT_LEN: usize = 60;

pub(crate) fn section_page(
    section: &Section,
    defaults: &dyn DynSection,
    types: &[TypeDecl],
) -> String {
    let mut page = String::new();
    page.push_str(&format!("# {}\n\n", section.file));
    page.push_str(&format!(
        "`{CONFIG_DIR}/{}.ron`, the `{}` section. Generated from `src/schema`; do not edit.\n\n",
        section.file, section.struct_name
    ));
    let about = if section.docs.is_empty() {
        &section.module_docs
    } else {
        &section.docs
    };
    if !about.is_empty() {
        page.push_str(&about.join("\n"));
        page.push_str("\n\n");
    }
    page.push_str("## Keys\n\n| Key | Type | Default | What it does |\n|---|---|---|---|\n");
    let mut long_defaults = Vec::new();
    for field in &section.fields {
        let default = defaults
            .get_ron(&field.name)
            .unwrap_or_else(|error| format!("<{error}>"));
        let shown = if default.len() > INLINE_DEFAULT_LEN {
            long_defaults.push((field.name.clone(), default));
            "see below".to_owned()
        } else {
            format!("`{default}`")
        };
        page.push_str(&format!(
            "| `{}` | `{}` | {shown} | {} |\n",
            field.name,
            field.ty.replace('|', "\\|"),
            docs::one_line(&field.docs)
        ));
    }
    page.push('\n');
    for (name, default) in long_defaults {
        page.push_str(&format!("Default `{name}`:\n\n```ron\n{default}\n```\n\n"));
    }
    let referenced = referenced_types(section, types);
    if !referenced.is_empty() {
        page.push_str("## Types\n\n");
        for decl in referenced {
            page.push_str(&type_section(decl));
        }
    }
    page.push_str("## Example\n\nThe file with every key at its default:\n\n```ron\n(\n");
    for field in &section.fields {
        if let Ok(default) = defaults.get_ron(&field.name) {
            page.push_str(&format!("    {}: {default},\n", field.name));
        }
    }
    page.push_str(")\n```\n");
    page
}

/// The types a section's keys mention, transitively, in first-mention order.
fn referenced_types<'a>(section: &Section, types: &'a [TypeDecl]) -> Vec<&'a TypeDecl> {
    let mut queue: Vec<&str> = section
        .fields
        .iter()
        .flat_map(|field| field.type_names.iter().map(String::as_str))
        .collect();
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    while !queue.is_empty() {
        let name = queue.remove(0);
        if !seen.insert(name) {
            continue;
        }
        if let Some(decl) = types.iter().find(|decl| decl.name == name) {
            found.push(decl);
            queue.extend(decl.mentioned_types());
        }
    }
    found
}

fn type_section(decl: &TypeDecl) -> String {
    let mut text = format!("### `{}`\n\n", decl.name);
    if !decl.docs.is_empty() {
        text.push_str(&decl.docs.join("\n"));
        text.push_str("\n\n");
    }
    match &decl.kind {
        TypeKind::Enum(variants) => {
            text.push_str("| Variant | What it means |\n|---|---|\n");
            for variant in variants {
                let default = if variant.is_default { " (default)" } else { "" };
                text.push_str(&format!(
                    "| `{}{}`{default} | {} |\n",
                    variant.name,
                    variant.payload.replace('|', "\\|"),
                    docs::one_line(&variant.docs)
                ));
            }
        }
        TypeKind::Struct(fields) => {
            text.push_str("| Field | Type | What it does |\n|---|---|---|\n");
            for field in fields {
                text.push_str(&format!(
                    "| `{}` | `{}` | {} |\n",
                    field.name,
                    field.ty.replace('|', "\\|"),
                    docs::one_line(&field.docs)
                ));
            }
        }
    }
    text.push('\n');
    text
}

pub(crate) fn index_page(sections: &[Section]) -> String {
    let mut page = String::from(
        "# CrownOS settings reference\n\n\
         Every setting is one RON file under `~/.config/crownos/`, named after its section. \
         Apps watch their file and apply a change as soon as it is saved, so editing the file is \
         the same as using the settings panel. Generated from `src/schema`; do not edit.\n\n\
         | Section | File | About | Read by |\n|---|---|---|---|\n",
    );
    for section in sections {
        let about = docs::first_paragraph(if section.docs.is_empty() {
            &section.module_docs
        } else {
            &section.docs
        });
        page.push_str(&format!(
            "| [{}]({}.md) | `{}.ron` | {} | {} |\n",
            section.file,
            section.file,
            section.file,
            about.replace('|', "\\|"),
            readers(&section.module_docs).replace('|', "\\|")
        ));
    }
    page
}

/// The sentences of a module doc that name who reads or writes the file.
fn readers(module_docs: &[String]) -> String {
    let text = docs::one_line(module_docs);
    let sentences: Vec<&str> = text
        .split(". ")
        .filter(|sentence| {
            let lower = sentence.to_ascii_lowercase();
            [
                "consumer",
                "reads",
                "writes",
                "written",
                "follows",
                "applied by",
            ]
            .iter()
            .any(|hint| lower.contains(hint))
        })
        .collect();
    if sentences.is_empty() {
        return "every CrownOS app, through crownos-config".to_owned();
    }
    let joined = sentences.join(". ");
    if joined.ends_with('.') {
        joined
    } else {
        format!("{joined}.")
    }
}

/// Keeps the generated Markdown free of the doc-link syntax that only rustdoc resolves.
pub(crate) fn plain_links(text: &str) -> String {
    text.replace("[`", "`").replace("`]", "`")
}
