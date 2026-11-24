//! `cargo xtask config-docs [--check]`: the settings reference under `docs/config/`, one page
//! per section, rendered from the schema sources so it cannot drift from them.
//! `cargo xtask show` and `cargo xtask check-defaults`: see [`effective_config`].

mod config_docs;
mod effective_config;
mod model;
mod parse;
mod render;
mod sections;
mod util;

use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

const USAGE: &str =
    "usage: xtask config-docs [--check] | show [<section>...] | check-defaults <directory>";

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
        Some("show") => effective_config::show(&arguments.collect::<Vec<_>>()),
        Some("check-defaults") => match arguments.next() {
            Some(directory) => effective_config::check_defaults(Path::new(&directory)),
            None => bail!(USAGE),
        },
        _ => bail!(USAGE),
    }
}

fn workspace_root() -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("xtask has no parent directory"))
}
