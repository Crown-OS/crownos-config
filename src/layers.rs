//! The files that give a section its value. From the lowest to the highest:
//! the `= default` of each field, the distro defaults that the `crownos-base`
//! package installs, the defaults of the machine, and the user's own file. The
//! user's file keeps only the keys that the user changed, so a system update
//! that changes a default reaches every user who did not change that key.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::util::path_for;

/// Replaces the default layer directories, lowest first, separated like `PATH`.
/// An empty value removes all the default layers.
pub const CONFIG_LAYERS_ENV: &str = "CROWN_CONFIG_LAYERS";

/// Replaces [`DEFAULTS_HISTORY_DIR`].
pub const CONFIG_HISTORY_ENV: &str = "CROWN_CONFIG_HISTORY";

/// Owned by the `crownos-base` package and replaced by each system update.
pub const DISTRO_DEFAULTS_DIR: &str = "/usr/share/crownos/defaults";

/// Defaults of this machine. No package owns them.
pub const MACHINE_DEFAULTS_DIR: &str = "/etc/crownos/defaults";

/// `<version>/<section>.ron`: the distro defaults of each earlier release. A
/// user file that an earlier release copied or wrote out in full is reduced
/// to the keys that differ from all of them.
pub const DEFAULTS_HISTORY_DIR: &str = "/usr/share/crownos/defaults-history";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionFiles {
    /// Lowest first.
    pub default_layers: Vec<PathBuf>,
    pub user: PathBuf,
    pub earlier_release_defaults: Vec<PathBuf>,
}

impl SectionFiles {
    pub fn for_section(section: &str) -> Self {
        let file_name = format!("{section}.ron");
        let default_layers = layer_directories()
            .into_iter()
            .map(|directory| directory.join(&file_name))
            .collect();
        Self {
            default_layers,
            user: path_for(section),
            earlier_release_defaults: earlier_release_files(&history_directory(), &file_name),
        }
    }
}

fn layer_directories() -> Vec<PathBuf> {
    match std::env::var_os(CONFIG_LAYERS_ENV) {
        Some(list) => split_directory_list(&list),
        None => vec![DISTRO_DEFAULTS_DIR.into(), MACHINE_DEFAULTS_DIR.into()],
    }
}

fn split_directory_list(list: &OsString) -> Vec<PathBuf> {
    std::env::split_paths(list)
        .filter(|directory| !directory.as_os_str().is_empty())
        .collect()
}

fn history_directory() -> PathBuf {
    std::env::var_os(CONFIG_HISTORY_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| DEFAULTS_HISTORY_DIR.into())
}

fn earlier_release_files(history: &Path, file_name: &str) -> Vec<PathBuf> {
    let Ok(releases) = std::fs::read_dir(history) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = releases
        .filter_map(Result::ok)
        .map(|release| release.path().join(file_name))
        .filter(|file| file.is_file())
        .collect();
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_layer_list_gives_no_layers() {
        assert!(split_directory_list(&OsString::new()).is_empty());
    }

    #[test]
    fn layer_directories_keep_their_order() {
        let list = std::env::join_paths(["/b", "/a"]).unwrap_or_default();
        assert_eq!(
            split_directory_list(&list),
            [PathBuf::from("/b"), PathBuf::from("/a")]
        );
    }

    #[test]
    fn a_missing_history_directory_gives_no_files() {
        assert!(earlier_release_files(Path::new("/nonexistent/history"), "dock.ron").is_empty());
    }
}
