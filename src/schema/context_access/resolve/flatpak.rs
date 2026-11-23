//! Flatpak apps that the user installed. Flatpak names an app by its id only, and a user can
//! install any id under `$HOME`, also the id of a system app. Such an app gets a key of its own,
//! which is session-only, so it never shares a grant with the system app.
//!
//! Limit: `.flatpak-info` is in the sandbox, so a host process that opens its own security
//! context can forge it and name a system install. This keeps honest user installs apart; it
//! does not prove an identity.

use std::path::{Path, PathBuf};

use super::super::app_key::is_controlled_by_root;
use super::AppKey;

/// The engine name that Flatpak gives `wp_security_context_v1`.
pub const FLATPAK_ENGINE: &str = "org.flatpak";
/// The engine of a key for a Flatpak app that root does not control.
pub const USER_FLATPAK_ENGINE: &str = "org.flatpak.user";

const INSTANCE_GROUP: &str = "[Instance]";
const APP_PATH_KEY: &str = "app-path=";

impl AppKey {
    /// The key of a sandboxed client whose process is `pid`. For Flatpak, the app's files must
    /// be outside `home` and controlled by root, as `/proc/<pid>/root/.flatpak-info` names them;
    /// otherwise, or when that file cannot be read, the key has [`USER_FLATPAK_ENGINE`].
    pub fn of_sandboxed_process(engine: &str, app_id: &str, pid: i32, home: Option<&Path>) -> Self {
        let info = (engine == FLATPAK_ENGINE)
            .then(|| std::fs::read_to_string(format!("/proc/{pid}/root/.flatpak-info")).ok())
            .flatten();
        sandboxed_key(engine, app_id, info.as_deref(), home, is_controlled_by_root)
    }
}

fn sandboxed_key(
    engine: &str,
    app_id: &str,
    flatpak_info: Option<&str>,
    home: Option<&Path>,
    root_controls: impl Fn(&Path) -> bool,
) -> AppKey {
    let system_install = flatpak_info.and_then(app_path).is_some_and(|path| {
        !home.is_some_and(|home| path.starts_with(home))
            && root_controls(&path)
            && path.parent().is_some_and(&root_controls)
    });
    let engine = if engine == FLATPAK_ENGINE && !system_install {
        USER_FLATPAK_ENGINE
    } else {
        engine
    };
    AppKey::Sandboxed {
        engine: engine.to_owned(),
        app_id: app_id.to_owned(),
    }
}

fn app_path(info: &str) -> Option<PathBuf> {
    let mut in_instance = false;
    for line in info.lines().map(str::trim) {
        if line.starts_with('[') {
            in_instance = line == INSTANCE_GROUP;
        } else if in_instance && let Some(path) = line.strip_prefix(APP_PATH_KEY) {
            let path = PathBuf::from(path.trim());
            return path.is_absolute().then_some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(app_path: &str) -> String {
        format!("[Application]\nname=org.mozilla.firefox\n\n[Instance]\napp-path={app_path}\n")
    }

    fn key(info: Option<&str>, root_controls: bool) -> AppKey {
        sandboxed_key(
            FLATPAK_ENGINE,
            "org.mozilla.firefox",
            info,
            Some(Path::new("/home/user")),
            |_| root_controls,
        )
    }

    #[test]
    fn a_system_install_keeps_the_flatpak_key() {
        let system = info("/var/lib/flatpak/app/org.mozilla.firefox/x86_64/stable/abc/files");
        let key = key(Some(&system), true);
        assert_eq!(key.to_string(), "sandbox:org.flatpak/org.mozilla.firefox");
        assert!(!key.is_session_only(None));
    }

    #[test]
    fn a_user_install_with_a_system_app_id_gets_a_session_only_key() {
        let user = info("/home/user/.local/share/flatpak/app/org.mozilla.firefox/current/files");
        for key in [
            key(Some(&user), true),
            key(None, true),
            key(Some(&info("/var/lib/x")), false),
        ] {
            assert_eq!(
                key.to_string(),
                "sandbox:org.flatpak.user/org.mozilla.firefox"
            );
            assert!(key.is_session_only(None));
        }
    }

    #[test]
    fn a_user_install_never_gets_raw_engine_or_policy_admin() {
        use crate::schema::{Scope, allowed_scopes};
        let user = key(None, true);
        let scopes = allowed_scopes(&user);
        for scope in [Scope::Raw, Scope::Engine, Scope::PolicyAdmin] {
            assert!(!scopes.contains(scope), "{scope:?}");
        }
        assert!(
            allowed_scopes(&key(Some(&info("/var/lib/flatpak/app/x")), true)).contains(Scope::Raw)
        );
    }

    #[test]
    fn a_forged_flatpak_info_naming_a_system_install_is_believed() {
        let forged =
            "[Instance]\napp-path=/var/lib/flatpak/app/org.bank.App/x86_64/stable/f/files\n";
        let claimed = sandboxed_key(
            FLATPAK_ENGINE,
            "org.bank.App",
            Some(forged),
            Some(Path::new("/home/user")),
            |_| true,
        );
        assert_eq!(claimed.to_string(), "sandbox:org.flatpak/org.bank.App");
        assert!(
            claimed.is_sandboxed(),
            "the key is the claim: a host process can forge it, so callers must not treat it as proof"
        );
    }

    #[test]
    fn other_engines_are_unchanged() {
        let key = sandboxed_key("org.snapcraft", "firefox", None, None, |_| false);
        assert_eq!(key.to_string(), "sandbox:org.snapcraft/firefox");
    }
}
