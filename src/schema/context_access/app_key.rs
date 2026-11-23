use std::borrow::Cow;
use std::fmt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use thiserror::Error;

const SANDBOX: &str = "sandbox:";
const SPAWNED: &str = "spawned:";
const EXECUTABLE: &str = "exe:";
const INTERPRETED: &str = "interp:";

/// Who an application is, in the stable form a grant is stored under.
///
/// The text form is `sandbox:<engine>/<app_id>`, `spawned:<component>`,
/// `exe:<path>` or `interp:<interpreter>:<script>`. A `%`, and the separator
/// that follows a part, are written as `%25`, `%2F` or `%3A` inside the
/// engine and the interpreter, so every part may contain any character.
///
/// An executable key holds the path only. The compositor checks the device
/// and inode of the file when the client connects.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AppKey {
    /// A client in a sandbox, as `wp_security_context_v1` names it.
    Sandboxed { engine: String, app_id: String },
    /// A CrownOS component that the compositor or systemd started.
    Spawned { component: String },
    /// An unsandboxed native executable.
    HostExecutable { path: PathBuf },
    /// A script run by an interpreter, such as `python3` or `node`.
    Interpreted {
        interpreter: PathBuf,
        script: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AppKeyError {
    #[error("{0:?} does not start with sandbox:, spawned:, exe: or interp:")]
    UnknownKind(String),
    #[error("{0:?} has no separator between its parts")]
    MissingSeparator(String),
    #[error("{0:?} has an empty part")]
    EmptyPart(String),
    #[error("{0:?} is not an absolute path")]
    RelativePath(String),
    #[error("{0:?} has an escape other than %25, %2F or %3A")]
    BadEscape(String),
}

impl AppKey {
    /// A script, or an executable the user can change, gets grants for this
    /// session only and never the scopes that capture or control others.
    /// The user can change an executable that is under `home`, or that is
    /// not owned by root, or that group or others can write, or whose
    /// directory is such. An executable that cannot be checked counts too.
    pub fn is_session_only(&self, home: Option<&Path>) -> bool {
        match self {
            Self::Interpreted { .. } => true,
            Self::HostExecutable { path } => {
                home.is_some_and(|home| path.starts_with(home))
                    || !is_controlled_by_root(path)
                    || path
                        .parent()
                        .is_none_or(|directory| !is_controlled_by_root(directory))
            }
            Self::Sandboxed { .. } | Self::Spawned { .. } => false,
        }
    }

    /// The name the user knows the app by: its app id, component, program
    /// or script.
    pub fn display_name(&self) -> String {
        let path = match self {
            Self::Sandboxed { app_id, .. } => return app_id.clone(),
            Self::Spawned { component } => return component.clone(),
            Self::HostExecutable { path } => path,
            Self::Interpreted { script, .. } => script,
        };
        path.file_name().map_or_else(
            || self.to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    }

    pub const fn is_sandboxed(&self) -> bool {
        matches!(self, Self::Sandboxed { .. })
    }
}

impl fmt::Display for AppKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sandboxed { engine, app_id } => {
                write!(formatter, "{SANDBOX}{}/{app_id}", escape(engine, '/'))
            }
            Self::Spawned { component } => write!(formatter, "{SPAWNED}{component}"),
            Self::HostExecutable { path } => write!(formatter, "{EXECUTABLE}{}", path.display()),
            Self::Interpreted {
                interpreter,
                script,
            } => write!(
                formatter,
                "{INTERPRETED}{}:{}",
                escape(&interpreter.to_string_lossy(), ':'),
                script.display()
            ),
        }
    }
}

impl FromStr for AppKey {
    type Err = AppKeyError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let missing_separator = || AppKeyError::MissingSeparator(text.to_owned());
        if let Some(rest) = text.strip_prefix(SANDBOX) {
            let (engine, app_id) = rest.split_once('/').ok_or_else(missing_separator)?;
            return Ok(Self::Sandboxed {
                engine: unescape(non_empty(engine, text)?, text)?.into_owned(),
                app_id: non_empty(app_id, text)?.to_owned(),
            });
        }
        if let Some(component) = text.strip_prefix(SPAWNED) {
            return Ok(Self::Spawned {
                component: non_empty(component, text)?.to_owned(),
            });
        }
        if let Some(path) = text.strip_prefix(EXECUTABLE) {
            return Ok(Self::HostExecutable {
                path: absolute(path)?,
            });
        }
        if let Some(rest) = text.strip_prefix(INTERPRETED) {
            let (interpreter, script) = rest.split_once(':').ok_or_else(missing_separator)?;
            return Ok(Self::Interpreted {
                interpreter: absolute(&unescape(interpreter, text)?)?,
                script: absolute(script)?,
            });
        }
        Err(AppKeyError::UnknownKind(text.to_owned()))
    }
}

fn is_controlled_by_root(path: &Path) -> bool {
    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.uid() == 0 && metadata.mode() & 0o022 == 0)
}

fn non_empty<'a>(part: &'a str, key: &str) -> Result<&'a str, AppKeyError> {
    if part.is_empty() {
        return Err(AppKeyError::EmptyPart(key.to_owned()));
    }
    Ok(part)
}

fn absolute(path: &str) -> Result<PathBuf, AppKeyError> {
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err(AppKeyError::RelativePath(path.display().to_string()));
    }
    Ok(path)
}

fn escape(part: &str, separator: char) -> Cow<'_, str> {
    if !part.contains(['%', separator]) {
        return Cow::Borrowed(part);
    }
    let mut escaped = String::with_capacity(part.len() + 4);
    for character in part.chars() {
        match character {
            '%' => escaped.push_str("%25"),
            '/' if separator == '/' => escaped.push_str("%2F"),
            ':' if separator == ':' => escaped.push_str("%3A"),
            other => escaped.push(other),
        }
    }
    Cow::Owned(escaped)
}

fn unescape<'a>(part: &'a str, key: &str) -> Result<Cow<'a, str>, AppKeyError> {
    if !part.contains('%') {
        return Ok(Cow::Borrowed(part));
    }
    let mut unescaped = String::with_capacity(part.len());
    let mut rest = part;
    while let Some(start) = rest.find('%') {
        unescaped.push_str(&rest[..start]);
        let decoded = match rest.get(start..start + 3) {
            Some("%25") => '%',
            Some("%2F" | "%2f") => '/',
            Some("%3A" | "%3a") => ':',
            _ => return Err(AppKeyError::BadEscape(key.to_owned())),
        };
        unescaped.push(decoded);
        rest = &rest[start + 3..];
    }
    unescaped.push_str(rest);
    Ok(Cow::Owned(unescaped))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(key: AppKey) {
        let text = key.to_string();
        assert_eq!(text.parse::<AppKey>(), Ok(key), "through {text}");
    }

    #[test]
    fn every_kind_survives_format_and_parse() {
        round_trip(AppKey::Sandboxed {
            engine: "flatpak".into(),
            app_id: "org.example.Notes".into(),
        });
        round_trip(AppKey::Spawned {
            component: "crown-contextd".into(),
        });
        round_trip(AppKey::HostExecutable {
            path: "/usr/bin/crownos-computer-use".into(),
        });
        round_trip(AppKey::Interpreted {
            interpreter: "/usr/bin/python3".into(),
            script: "/home/user/agent.py".into(),
        });
    }

    #[test]
    fn separators_and_percent_signs_inside_parts_survive() {
        round_trip(AppKey::Sandboxed {
            engine: "snap/strict%".into(),
            app_id: "org/example:Notes%41".into(),
        });
        round_trip(AppKey::Interpreted {
            interpreter: "/opt/py:3/bin/py%thon".into(),
            script: "/srv/a:b/c%3A.py".into(),
        });
        round_trip(AppKey::HostExecutable {
            path: "/opt/a:b/run%".into(),
        });
    }

    #[test]
    fn the_text_form_is_the_documented_one() {
        let key = AppKey::Interpreted {
            interpreter: "/usr/bin/node".into(),
            script: "/srv/bot.js".into(),
        };
        assert_eq!(key.to_string(), "interp:/usr/bin/node:/srv/bot.js");
        assert_eq!(
            "sandbox:flatpak/org.example.Notes".parse(),
            Ok(AppKey::Sandboxed {
                engine: "flatpak".into(),
                app_id: "org.example.Notes".into(),
            })
        );
    }

    #[test]
    fn apps_are_named_by_what_the_user_knows() {
        let script = AppKey::Interpreted {
            interpreter: "/usr/bin/python3".into(),
            script: "/srv/bot.py".into(),
        };
        assert_eq!(script.display_name(), "bot.py");
        let program = AppKey::HostExecutable {
            path: "/usr/bin/agent".into(),
        };
        assert_eq!(program.display_name(), "agent");
    }

    #[test]
    fn malformed_keys_are_refused() {
        for text in [
            "",
            "flatpak:org.example",
            "sandbox:flatpak",
            "sandbox:/org.example",
            "sandbox:flatpak/",
            "spawned:",
            "exe:bin/sh",
            "interp:/usr/bin/python3",
            "interp:/usr/bin/py%zz:/a.py",
        ] {
            assert!(text.parse::<AppKey>().is_err(), "{text:?} parsed");
        }
    }

    #[test]
    fn scripts_and_executables_the_user_can_change_are_session_only() -> std::io::Result<()> {
        let script = AppKey::Interpreted {
            interpreter: "/usr/bin/python3".into(),
            script: "/usr/share/agent.py".into(),
        };
        let root_owned = AppKey::HostExecutable {
            path: "/usr/bin/env".into(),
        };
        let user_owned_path =
            std::env::temp_dir().join(format!("crownos-config-app-key-{}", std::process::id()));
        std::fs::write(&user_owned_path, b"")?;
        let user_owned = AppKey::HostExecutable {
            path: user_owned_path.clone(),
        };
        let missing = AppKey::HostExecutable {
            path: "/usr/bin/crownos-no-such-agent".into(),
        };

        assert!(script.is_session_only(None));
        if is_controlled_by_root(Path::new("/usr/bin/env"))
            && is_controlled_by_root(Path::new("/usr/bin"))
        {
            assert!(!root_owned.is_session_only(None));
        }
        assert!(root_owned.is_session_only(Some(Path::new("/usr"))));
        assert!(user_owned.is_session_only(None));
        assert!(missing.is_session_only(None));
        std::fs::remove_file(user_owned_path)
    }
}
