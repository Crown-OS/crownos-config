//! Launchers that start the real program of an app: a script or a small binary that `exec`s
//! another file. A window connects as the real program, so the key of a desktop entry's `Exec`
//! must name that program.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::super::app_key::is_controlled_by_root;

/// Where an administrator adds launchers. Only a file that root owns and that group and others
/// cannot write is read.
pub const LAUNCHER_FILE: &str = "/etc/crownos/app-launchers.ron";

/// The Arch Linux packages' launchers. Chromium and Google Chrome were measured; Brave
/// (brave-bin), Firefox and Vivaldi follow their package file lists.
const BUILT_IN_LAUNCHERS: [(&str, &str); 8] = [
    ("/usr/bin/chromium", "/usr/lib/chromium/chromium"),
    ("/usr/bin/google-chrome-stable", "/opt/google/chrome/chrome"),
    (
        "/opt/google/chrome/google-chrome",
        "/opt/google/chrome/chrome",
    ),
    ("/usr/bin/brave", "/opt/brave-bin/brave"),
    ("/opt/brave-bin/brave-browser", "/opt/brave-bin/brave"),
    ("/usr/bin/firefox", "/usr/lib/firefox/firefox"),
    ("/usr/bin/vivaldi-stable", "/opt/vivaldi/vivaldi-bin"),
    ("/opt/vivaldi/vivaldi", "/opt/vivaldi/vivaldi-bin"),
];

#[derive(Debug, Error)]
pub enum LauncherFileError {
    #[error("{0} is not owned by root, or group or others can write it")]
    NotRootControlled(PathBuf),
    #[error("cannot read {0}: {1}")]
    Unreadable(PathBuf, std::io::Error),
    #[error("{0} is not a map of paths: {1}")]
    Malformed(PathBuf, Box<ron::error::SpannedError>),
    #[error("{0} maps a path that is not absolute")]
    RelativePath(PathBuf),
}

/// Launcher → real program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherMap {
    programs: HashMap<PathBuf, PathBuf>,
}

impl LauncherMap {
    pub fn built_in() -> Self {
        Self {
            programs: BUILT_IN_LAUNCHERS
                .iter()
                .map(|(launcher, program)| (PathBuf::from(launcher), PathBuf::from(program)))
                .collect(),
        }
    }

    /// The built-in launchers and those of [`LAUNCHER_FILE`], when it exists and root controls
    /// it.
    ///
    /// # Errors
    ///
    /// The file exists but is not root-controlled or not a map of absolute paths. The caller
    /// keeps [`Self::built_in`] then.
    pub fn installed() -> Result<Self, LauncherFileError> {
        let file = Path::new(LAUNCHER_FILE);
        if !file.exists() {
            return Ok(Self::built_in());
        }
        Self::built_in().extended_by(file)
    }

    /// These launchers and those of `file`.
    ///
    /// # Errors
    ///
    /// As [`Self::installed`].
    pub fn extended_by(mut self, file: &Path) -> Result<Self, LauncherFileError> {
        let directory_controlled = file.parent().is_some_and(is_controlled_by_root);
        if !is_controlled_by_root(file) || !directory_controlled {
            return Err(LauncherFileError::NotRootControlled(file.to_path_buf()));
        }
        let text = std::fs::read_to_string(file)
            .map_err(|error| LauncherFileError::Unreadable(file.to_path_buf(), error))?;
        self.add_entries(file, &text)?;
        Ok(self)
    }

    fn add_entries(&mut self, file: &Path, text: &str) -> Result<(), LauncherFileError> {
        let entries: HashMap<PathBuf, PathBuf> = ron::from_str(text)
            .map_err(|error| LauncherFileError::Malformed(file.to_path_buf(), Box::new(error)))?;
        if entries
            .iter()
            .any(|(launcher, program)| !launcher.is_absolute() || !program.is_absolute())
        {
            return Err(LauncherFileError::RelativePath(file.to_path_buf()));
        }
        self.programs.extend(entries);
        Ok(())
    }

    /// The program that `executable` starts: itself when it is not a known launcher.
    pub fn real_program<'path>(&'path self, executable: &'path Path) -> &'path Path {
        self.programs
            .get(executable)
            .map_or(executable, PathBuf::as_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_launcher_gives_its_real_program() {
        let launchers = LauncherMap::built_in();
        assert_eq!(
            launchers.real_program(Path::new("/usr/bin/chromium")),
            Path::new("/usr/lib/chromium/chromium")
        );
        assert_eq!(
            launchers.real_program(Path::new("/usr/bin/nautilus")),
            Path::new("/usr/bin/nautilus")
        );
    }

    #[test]
    fn a_file_the_user_controls_is_refused() {
        let directory = std::env::temp_dir().join(format!("launchers-{}", std::process::id()));
        std::fs::create_dir_all(&directory).ok();
        let file = directory.join("app-launchers.ron");
        std::fs::write(&file, r#"{"/usr/bin/bank": "/home/user/evil"}"#).ok();
        let writable_by_others = std::os::unix::fs::PermissionsExt::from_mode(0o666);
        std::fs::set_permissions(&file, writable_by_others).ok();
        let extended = LauncherMap::built_in().extended_by(&file);
        assert!(matches!(
            extended,
            Err(LauncherFileError::NotRootControlled(_))
        ));
        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn entries_must_be_absolute_paths() {
        let mut launchers = LauncherMap::built_in();
        let file = Path::new(LAUNCHER_FILE);
        assert!(
            launchers
                .add_entries(file, r#"{"/usr/bin/app": "/opt/app/app"}"#)
                .is_ok()
        );
        assert_eq!(
            launchers.real_program(Path::new("/usr/bin/app")),
            Path::new("/opt/app/app")
        );
        assert!(matches!(
            launchers.add_entries(file, r#"{"app": "/opt/app/app"}"#),
            Err(LauncherFileError::RelativePath(_))
        ));
        assert!(matches!(
            launchers.add_entries(file, "not ron"),
            Err(LauncherFileError::Malformed(..))
        ));
    }
}
