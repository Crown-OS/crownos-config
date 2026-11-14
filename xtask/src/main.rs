//! `cargo xtask config-docs [--check]`: the settings reference under `docs/config/`, one page
//! per section, rendered from the schema sources so it cannot drift from them.

mod config_docs;
mod model;
mod parse;
mod render;
mod sections;
mod util;

use std::path::PathBuf;

use anyhow::{Result, bail};

fn main() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    match arguments.next().as_deref() {
        Some("config-docs") => {
            let check = match arguments.next().as_deref() {
                None => false,
                Some("--check") => true,
                Some(other) => {
                    bail!("unknown argument {other}; usage: xtask config-docs [--check]")
                }
            };
            config_docs::run(&workspace_root()?, check)
        }
        _ => bail!("usage: xtask config-docs [--check]"),
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("xtask has no parent directory"))
}
