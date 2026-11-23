use std::path::PathBuf;

use super::{AppKey, ContextAccess, Expiry, Scope, ScopeSet};
use crate::schema::{AccessDecision, Agents};

/// `agents.ron` names applications by their app id only. Their grants are
/// copied under this sandbox engine.
pub const MIGRATED_SANDBOX_ENGINE: &str = "flatpak";

const CONTEXT_ENGINE_EXECUTABLE: &str = "/usr/bin/crown-contextd";
const CONTEXT_ENGINE_COMPONENT: &str = "crown-contextd";

/// What a trusted executable of `agents.ron` could do before: see windows,
/// pause, control windows and drive agent cursors. Never pixels.
const TRUSTED_EXECUTABLE_SCOPES: ScopeSet = ScopeSet::EMPTY
    .with(Scope::Metadata)
    .with(Scope::Text)
    .with(Scope::Pause)
    .with(Scope::WaylandWindowControl)
    .with(Scope::WaylandAgentCursor);

impl ContextAccess {
    /// Copies the grants of `agents.ron` into this section, one time only:
    ///
    /// - `/usr/bin/crown-contextd` becomes `spawned:crown-contextd` with
    ///   [`Scope::Engine`];
    /// - each other trusted executable gets an `exe:` grant with metadata,
    ///   text, pause, window control and agent cursor;
    /// - each allowed app id gets a `sandbox:` grant with [`Scope::Metadata`];
    /// - each refused app id becomes a `sandbox:` refusal.
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
            if path == CONTEXT_ENGINE_EXECUTABLE {
                let key = AppKey::Spawned {
                    component: CONTEXT_ENGINE_COMPONENT.to_owned(),
                };
                self.grant_unless_decided(&key, ScopeSet::EMPTY.with(Scope::Engine));
            } else {
                let key = AppKey::HostExecutable {
                    path: PathBuf::from(path),
                };
                self.grant_unless_decided(&key, TRUSTED_EXECUTABLE_SCOPES);
            }
        }
        for (app_id, decision) in &agents.app_access {
            let key = AppKey::Sandboxed {
                engine: MIGRATED_SANDBOX_ENGINE.to_owned(),
                app_id: app_id.clone(),
            };
            match decision {
                AccessDecision::Allow => {
                    self.grant_unless_decided(&key, ScopeSet::EMPTY.with(Scope::Metadata));
                }
                AccessDecision::Deny if !self.is_decided(&key) => self.deny(&key),
                AccessDecision::Deny => {}
            }
        }
        true
    }

    fn is_decided(&self, key: &AppKey) -> bool {
        let text = key.to_string();
        self.grants.contains_key(&text) || self.denied.contains(&text)
    }

    fn grant_unless_decided(&mut self, key: &AppKey, scopes: ScopeSet) {
        if !self.is_decided(key) {
            self.grant(key, scopes, Expiry::Never);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::SystemTime;

    use super::super::Grant;
    use super::*;

    fn agents() -> Agents {
        Agents {
            trusted_executables: vec![
                "/usr/bin/crown-contextd".to_owned(),
                "/usr/bin/crownos-computer-use".to_owned(),
            ],
            ask_for_app_access: true,
            app_access: BTreeMap::from([
                ("org.example.Notes".to_owned(), AccessDecision::Allow),
                ("org.example.Spy".to_owned(), AccessDecision::Deny),
            ]),
        }
    }

    #[test]
    fn the_context_engine_becomes_a_spawned_component_with_the_engine_scope() {
        let mut access = ContextAccess::default();
        assert!(access.migrate_agents(&agents()));
        let contextd = AppKey::Spawned {
            component: "crown-contextd".into(),
        };
        assert_eq!(
            access.grants["spawned:crown-contextd"].scopes,
            [Scope::Engine]
        );
        let scopes = access.scopes_for(&contextd, SystemTime::now());
        assert!(scopes.contains(Scope::Engine));
        assert!(scopes.contains(Scope::Metadata));
        assert!(!access.grants.contains_key("exe:/usr/bin/crown-contextd"));
    }

    #[test]
    fn other_trusted_executables_get_agent_scopes_without_raw_engine_or_admin() {
        let mut access = ContextAccess::default();
        access.migrate_agents(&agents());
        let computer_use = AppKey::HostExecutable {
            path: "/usr/bin/crownos-computer-use".into(),
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
        assert_eq!(engines, ["spawned:crown-contextd"]);
    }

    #[test]
    fn app_answers_become_sandbox_grants_with_metadata_only_and_refusals() {
        let mut access = ContextAccess::default();
        access.migrate_agents(&agents());
        assert_eq!(
            access.grants["sandbox:flatpak/org.example.Notes"],
            Grant {
                scopes: vec![Scope::Metadata],
                expires: Expiry::Never,
            }
        );
        assert_eq!(access.denied, ["sandbox:flatpak/org.example.Spy"]);
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
