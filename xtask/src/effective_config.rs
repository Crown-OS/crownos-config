//! `cargo xtask show [<section>...]`: the effective value of each section, every
//! key included, for the config of `CROWN_CONFIG_DIR` and the layers of
//! `CROWN_CONFIG_LAYERS`. With no section, every section of the schema. The
//! update test compares this output for an updated and a freshly installed
//! system.
//!
//! `cargo xtask check-defaults <directory>`: validates each `<section>.ron` in
//! a default layer directory against the schema.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result, bail};
use crownos_config::schema::{SECTIONS, read_section};
use crownos_config::validate_default_layer;

pub fn show(sections: &[String]) -> Result<()> {
    let selected: Vec<&str> = if sections.is_empty() {
        SECTIONS.to_vec()
    } else {
        sections.iter().map(String::as_str).collect()
    };
    let mut output = String::new();
    for section in selected {
        let text = read_section(section, |fields| {
            let mut text = format!("{section}: (\n");
            for key in fields.key_names() {
                let _ = writeln!(text, "    {key}: {},", fields.get_ron(key)?);
            }
            text.push_str(")\n");
            Ok(text)
        })
        .with_context(|| format!("read section {section}"))?;
        output.push_str(&text);
    }
    print!("{output}");
    Ok(())
}

pub fn check_defaults(directory: &Path) -> Result<()> {
    let entries =
        std::fs::read_dir(directory).with_context(|| format!("read {}", directory.display()))?;
    let mut failures = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("ron") {
            continue;
        }
        let Some(section) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let text =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if let Err(error) = validate_default_layer(section, &text) {
            failures.push(format!("{}: {error}", path.display()));
        }
    }
    if !failures.is_empty() {
        bail!("invalid default layer files:\n{}", failures.join("\n"));
    }
    Ok(())
}
