//! The files that give a section its value. From the lowest to the highest:
//! the `= default` of each field, the distro defaults that the `crownos-base`
//! package installs, the defaults of the machine, and the user's own file. The
//! user's file sets only the keys that the user changed, so a system update
//! that changes a default reaches every user who did not change that key. The
//! other keys are in the file as comments with their current default, so the
//! user can change one by removing the `//`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::config_dir;
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

/// A file in [`DEFAULTS_HISTORY_DIR`] that allows the reduction of the full
/// files that earlier releases wrote. The first release with layers does not
/// ship it: a rollback from that release to one without layers then still
/// finds the full files, because the older release has no default layers.
pub const REDUCTION_FLAG_FILE: &str = "reduce-home-files";

/// The directory in the config directory that holds one empty file for each
/// section whose user file is reduced to the user's changes, or had no file.
const REDUCED_SECTIONS_DIR: &str = ".layered";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionFiles {
    /// Lowest first.
    pub default_layers: Vec<PathBuf>,
    pub user: PathBuf,
    /// Exists when the user file needs no reduction: a file that the user
    /// writes after that keeps every key.
    pub reduced_marker: PathBuf,
    pub defaults_history: PathBuf,
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
            reduced_marker: config_dir().join(REDUCED_SECTIONS_DIR).join(section),
            defaults_history: history_directory(),
        }
    }

    /// The distro files of this section in earlier releases. Read only for a
    /// reduction, because the history has one directory for each release.
    pub fn earlier_release_defaults(&self) -> Vec<PathBuf> {
        let Some(file_name) = self.user.file_name() else {
            return Vec::new();
        };
        earlier_release_files(&self.defaults_history, Path::new(file_name))
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

fn earlier_release_files(history: &Path, file_name: &Path) -> Vec<PathBuf> {
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
        assert!(
            earlier_release_files(Path::new("/nonexistent/history"), Path::new("dock.ron"))
                .is_empty()
        );
    }
}
