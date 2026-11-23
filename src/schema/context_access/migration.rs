use std::path::PathBuf;

use super::{AppKey, CONTEXT_ENGINE_EXECUTABLE, ContextAccess, Expiry, Scope, ScopeSet};
use crate::schema::Agents;

/// What a trusted executable of `agents.ron` could do before: see windows,
/// pause, control windows and drive agent cursors. Never pixels.
const TRUSTED_EXECUTABLE_SCOPES: ScopeSet = ScopeSet::EMPTY
    .with(Scope::Metadata)
    .with(Scope::Text)
    .with(Scope::Pause)
    .with(Scope::WaylandWindowControl)
    .with(Scope::WaylandAgentCursor);

impl ContextAccess {
    /// Copies the trusted executables of `agents.ron` into this section, one
    /// time only:
    ///
    /// - `/usr/bin/crown-contextd` gets an `exe:` grant with
    ///   [`Scope::Engine`], which it holds while root controls the file;
    /// - each other trusted executable gets an `exe:` grant with metadata,
    ///   text, pause, window control and agent cursor.
    ///
    /// `app_access` is not copied: it records which apps show their own UI
    /// to agents, not which apps may read context.
    ///
    /// The migration never grants [`Scope::Raw`] or [`Scope::PolicyAdmin`].
    /// An existing grant or refusal for the same key stays. `agents.ron` does
    /// not change, because the compositor still reads it. Returns whether
    /// this section changed; when it did, save it.
    pub fn migrate_agents(&mut self, agents: &Agents) -> bool {
        if self.migrated_agents {
            return false;
        }
        self.migrated_agents = true;
        for path in &agents.trusted_executables {
            let key = AppKey::HostExecutable {
                path: PathBuf::from(path),
            };
            let scopes = if path == CONTEXT_ENGINE_EXECUTABLE {
                ScopeSet::EMPTY.with(Scope::Engine)
            } else {
                TRUSTED_EXECUTABLE_SCOPES
            };
            self.grant_unless_decided(&key, scopes);
        }
        true
    }

    fn is_decided(&self, key: &AppKey) -> bool {
        let text = key.to_string();
        self.grants.contains_key(&text) || self.denied.contains(&text)
    }

    fn grant_unless_decided(&mut self, key: &AppKey, scopes: ScopeSet) {
        if !self.is_decided(key) {
            self.store_grant(key, scopes, Expiry::Never);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::SystemTime;

    use super::*;
    use crate::schema::AccessDecision;

    fn agents() -> Agents {
        Agents {
            trusted_executables: vec![
                "/usr/bin/crown-contextd".to_owned(),
                "/usr/bin/env".to_owned(),
            ],
            ask_for_app_access: true,
            app_access: BTreeMap::from([
                ("org.example.Notes".to_owned(), AccessDecision::Allow),
                ("org.example.Spy".to_owned(), AccessDecision::Deny),
            ]),
        }
    }

    #[test]
    fn the_context_engine_keeps_its_executable_key_with_the_engine_scope() {
        let mut access = ContextAccess::default();
        assert!(access.migrate_agents(&agents()));
        assert_eq!(
            access.grants["exe:/usr/bin/crown-contextd"].scopes,
            [Scope::Engine]
        );
        assert!(!access.grants.keys().any(|key| key.starts_with("spawned:")));
        let contextd = AppKey::HostExecutable {
            path: CONTEXT_ENGINE_EXECUTABLE.into(),
        };
        let root_controls_it = !contextd.is_session_only(dirs::home_dir().as_deref());
        let scopes = access.scopes_for(&contextd, SystemTime::now());
        assert_eq!(scopes.contains(Scope::Engine), root_controls_it);
    }

    #[test]
    fn other_trusted_executables_get_agent_scopes_without_raw_engine_or_admin() {
        let mut access = ContextAccess::default();
        access.migrate_agents(&agents());
        let computer_use = AppKey::HostExecutable {
            path: "/usr/bin/env".into(),
        };
        let scopes = access.scopes_for(&computer_use, SystemTime::now());
        assert_eq!(scopes, TRUSTED_EXECUTABLE_SCOPES);
        for never in [Scope::Raw, Scope::Engine, Scope::PolicyAdmin] {
            assert!(!scopes.contains(never), "{never}");
        }
    }

    #[test]
    fn the_migration_never_grants_raw_or_policy_admin() {
        let mut access = ContextAccess::default();
        access.migrate_agents(&Agents::default());
        access.migrate_agents(&agents());
        for grant in access.grants.values() {
            assert!(!grant.scopes.contains(&Scope::Raw));
            assert!(!grant.scopes.contains(&Scope::PolicyAdmin));
        }
        let engines = access
            .grants
            .iter()
            .filter(|(_, grant)| grant.scopes.contains(&Scope::Engine))
            .map(|(key, _)| key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(engines, ["exe:/usr/bin/crown-contextd"]);
    }

    #[test]
    fn a_trusted_executable_that_is_not_installed_keeps_a_lasting_grant() {
        let mut access = ContextAccess::default();
        access.migrate_agents(&Agents {
            trusted_executables: vec!["/usr/bin/crownos-not-installed".to_owned()],
            ..Agents::default()
        });
        assert_eq!(
            access.grants["exe:/usr/bin/crownos-not-installed"].expires,
            Expiry::Never
        );
        assert!(!access.forget_session_grants());
    }

    #[test]
    fn app_access_answers_are_not_copied() {
        let mut access = ContextAccess::default();
        access.migrate_agents(&agents());
        assert!(access.grants.keys().all(|key| !key.starts_with("sandbox:")));
        assert!(access.denied.is_empty());
    }

    #[test]
    fn the_migration_runs_once_and_keeps_earlier_answers() {
        let mut access = ContextAccess::default();
        access
            .denied
            .push("exe:/usr/bin/crownos-computer-use".to_owned());
        assert!(access.migrate_agents(&agents()));
        assert!(
            !access
                .grants
                .contains_key("exe:/usr/bin/crownos-computer-use")
        );

        access.grants.clear();
        access.denied.clear();
        assert!(!access.migrate_agents(&agents()));
        assert!(access.grants.is_empty());
        assert!(access.denied.is_empty());
    }
}
