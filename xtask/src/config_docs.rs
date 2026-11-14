//! Writes or checks `docs/config/`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::parse::read_schema;
use crate::render::{index_page, plain_links, section_page};
use crate::sections::defaults;

const OUT_DIR: &str = "docs/config";

pub(crate) fn run(root: &Path, check: bool) -> Result<()> {
    let schema = read_schema(&root.join("src/schema"))?;
    let defaults = defaults();
    let declared: Vec<&str> = schema.sections.iter().map(|s| s.file.as_str()).collect();
    let listed: Vec<&str> = defaults.iter().map(|d| d.section_name()).collect();
    for name in &declared {
        if !listed.contains(name) {
            bail!(
                "section {name} is declared in src/schema but missing from xtask/src/sections.rs"
            );
        }
    }
    for name in &listed {
        if !declared.contains(name) {
            bail!("xtask/src/sections.rs lists {name}, which src/schema does not declare");
        }
    }
    let mut pages: BTreeMap<String, String> = BTreeMap::new();
    for section in &schema.sections {
        let default = defaults
            .iter()
            .find(|d| d.section_name() == section.file)
            .context("checked above")?;
        pages.insert(
            format!("{}.md", section.file),
            plain_links(&section_page(section, default.as_ref(), &schema.types)),
        );
    }
    pages.insert(
        "index.md".to_owned(),
        plain_links(&index_page(&schema.sections)),
    );

    let out = root.join(OUT_DIR);
    if check {
        return verify(&out, &pages);
    }
    fs::create_dir_all(&out)?;
    for entry in fs::read_dir(&out)? {
        let path = entry?.path();
        let stale = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| !pages.contains_key(name));
        if stale {
            fs::remove_file(&path)?;
        }
    }
    for (name, text) in &pages {
        fs::write(out.join(name), text)?;
    }
    println!("wrote {} pages to {}", pages.len(), out.display());
    Ok(())
}

fn verify(out: &Path, pages: &BTreeMap<String, String>) -> Result<()> {
    let mut stale = Vec::new();
    for (name, text) in pages {
        match fs::read_to_string(out.join(name)) {
            Ok(existing) if existing == *text => {}
            _ => stale.push(name.clone()),
        }
    }
    if out.is_dir() {
        for entry in fs::read_dir(out)? {
            let name = entry?.file_name().to_string_lossy().into_owned();
            if !pages.contains_key(&name) {
                stale.push(format!("{name} (no longer generated)"));
            }
        }
    }
    if stale.is_empty() {
        println!("{} is up to date", out.display());
        return Ok(());
    }
    bail!(
        "{} is stale: {}\nrun `cargo xtask config-docs`",
        out.display(),
        stale.join(", ")
    );
}
