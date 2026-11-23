//! Which applications may use screen context, and how much of it.
//!
//! The consumers are [crownpositor], which gates the context, window control
//! and agent cursor protocols, and [crown-contextd], which gates its API.
//! Both read this file live and stop a client at once when its grant goes.
//!
//! A grant is stored under the text form of an [`AppKey`]. An application
//! without a grant gets the default of its kind: `sandboxed_default` for a
//! sandboxed client, `host_default` for every other client.
//!
//! [crownpositor]: https://github.com/crown-os/crownpositor
//! [crown-contextd]: https://github.com/crown-os/crownos-visual-context-engine

mod app_key;
mod migration;
mod scope;

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub use app_key::{AppKey, AppKeyError};
pub use migration::MIGRATED_SANDBOX_ENGINE;
pub use scope::{Scope, ScopeSet};

/// What happens when an application without a grant asks for context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirstUsePolicy {
    /// Refuse without a question.
    Deny,
    /// Ask the user on screen, if `ask_on_first_use` is on.
    Ask,
}

/// When a grant stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Expiry {
    /// The grant stays until the user removes it.
    #[default]
    Never,
    /// The grant stops when the user logs out. The compositor calls
    /// [`ContextAccess::forget_session_grants`] when a session starts.
    Session,
    /// The grant stops at this time, in seconds since the Unix epoch.
    At(u64),
}

impl Expiry {
    pub fn has_passed(self, now: SystemTime) -> bool {
        match self {
            Self::Never | Self::Session => false,
            Self::At(seconds) => now
                .duration_since(UNIX_EPOCH)
                .is_ok_and(|elapsed| elapsed.as_secs() >= seconds),
        }
    }
}

/// The scopes the user gave one application.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Grant {
    /// What the application may do.
    pub scopes: Vec<Scope>,
    /// When the grant stops.
    pub expires: Expiry,
}

/// The answer for one application at one time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDecision {
    Granted(ScopeSet),
    /// No grant yet: ask the user, then store the answer.
    Ask,
    Denied,
}

crate::section! {
    pub struct ContextAccess in "context_access", keys ContextAccessKey {
        /// Ask on screen the first time an application without a grant asks
        /// for context. Off, such an application is refused.
        pub ask_on_first_use as AskOnFirstUse: bool = true,

        /// What a sandboxed application without a grant gets.
        pub sandboxed_default as SandboxedDefault: FirstUsePolicy = FirstUsePolicy::Deny,

        /// What every other application without a grant gets.
        pub host_default as HostDefault: FirstUsePolicy = FirstUsePolicy::Ask,

        /// Grants by application key, such as `exe:/usr/bin/crown-contextd`
        /// or `sandbox:flatpak/org.example.Notes`.
        pub grants as Grants: BTreeMap<String, Grant> = BTreeMap::new(),

        /// Application keys the user refused. A refusal is stronger than a grant.
        pub denied as Denied: Vec<String> = Vec::new(),

        /// Minutes after which the user must authenticate again before an
        /// application gets screen pixels.
        pub raw_reauth_minutes as RawReauthMinutes: u32 = 5,

        /// Set when the grants of `agents.ron` were copied into this file,
        /// so that a grant the user removes later does not come back.
        pub migrated_agents as MigratedAgents: bool = false,
    }
}

impl ContextAccess {
    /// The decision for `key` at `now`: a refusal first, then a grant that has
    /// not expired, then the default of the key's kind. A granted set has the
    /// scopes that [`Scope::Engine`] includes.
    pub fn decide(&self, key: &AppKey, now: SystemTime) -> ContextDecision {
        let text = key.to_string();
        if self.denied.contains(&text) {
            return ContextDecision::Denied;
        }
        if let Some(grant) = self
            .grants
            .get(&text)
            .filter(|grant| !grant.expires.has_passed(now))
        {
            let scopes = grant.scopes.iter().collect::<ScopeSet>().with_included();
            return ContextDecision::Granted(if is_session_only(key) {
                scopes.without(Scope::Raw)
            } else {
                scopes
            });
        }
        let policy = if key.is_sandboxed() {
            self.sandboxed_default
        } else {
            self.host_default
        };
        match policy {
            FirstUsePolicy::Ask if self.ask_on_first_use => ContextDecision::Ask,
            FirstUsePolicy::Ask | FirstUsePolicy::Deny => ContextDecision::Denied,
        }
    }

    /// The scopes `key` holds at `now`; empty when it must ask or is refused.
    pub fn scopes_for(&self, key: &AppKey, now: SystemTime) -> ScopeSet {
        match self.decide(key, now) {
            ContextDecision::Granted(scopes) => scopes,
            ContextDecision::Ask | ContextDecision::Denied => ScopeSet::EMPTY,
        }
    }

    /// Stores a grant and removes an earlier refusal. A session-only key gets
    /// at most a session grant and never [`Scope::Raw`].
    pub fn grant(&mut self, key: &AppKey, scopes: ScopeSet, expires: Expiry) {
        let (scopes, expires) = if is_session_only(key) {
            let expires = match expires {
                Expiry::Never => Expiry::Session,
                other => other,
            };
            (scopes.without(Scope::Raw), expires)
        } else {
            (scopes, expires)
        };
        let text = key.to_string();
        self.denied.retain(|denied| *denied != text);
        self.grants.insert(
            text,
            Grant {
                scopes: scopes.iter().collect(),
                expires,
            },
        );
    }

    /// Removes the grant of `key` and remembers the refusal.
    pub fn deny(&mut self, key: &AppKey) {
        let text = key.to_string();
        self.grants.remove(&text);
        if !self.denied.contains(&text) {
            self.denied.push(text);
        }
    }

    /// Removes every session grant, and every grant of a session-only key.
    /// Returns whether something changed.
    pub fn forget_session_grants(&mut self) -> bool {
        let before = self.grants.len();
        self.grants.retain(|text, grant| {
            let session_only = text
                .parse::<AppKey>()
                .is_ok_and(|key| is_session_only(&key));
            grant.expires != Expiry::Session && !session_only
        });
        self.grants.len() != before
    }
}

fn is_session_only(key: &AppKey) -> bool {
    key.is_session_only(dirs::home_dir().as_deref())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn executable(path: &str) -> AppKey {
        AppKey::HostExecutable { path: path.into() }
    }

    fn sandboxed(app_id: &str) -> AppKey {
        AppKey::Sandboxed {
            engine: "flatpak".into(),
            app_id: app_id.into(),
        }
    }

    fn at(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    #[test]
    fn defaults_ask_host_apps_and_refuse_sandboxed_apps() {
        let access = ContextAccess::default();
        let now = at(0);
        assert_eq!(
            access.decide(&executable("/usr/bin/agent"), now),
            ContextDecision::Ask
        );
        assert_eq!(
            access.decide(&sandboxed("org.example.Notes"), now),
            ContextDecision::Denied
        );
        assert!(
            access
                .scopes_for(&executable("/usr/bin/agent"), now)
                .is_empty()
        );
    }

    #[test]
    fn without_first_use_questions_asking_becomes_a_refusal() {
        let access = ContextAccess {
            ask_on_first_use: false,
            ..ContextAccess::default()
        };
        assert_eq!(
            access.decide(&executable("/usr/bin/agent"), at(0)),
            ContextDecision::Denied
        );
    }

    #[test]
    fn a_grant_gives_its_scopes_until_it_expires() {
        let key = executable("/usr/bin/agent");
        let mut access = ContextAccess::default();
        let scopes = ScopeSet::EMPTY.with(Scope::Metadata).with(Scope::Raw);
        access.grant(&key, scopes, Expiry::At(100));
        assert_eq!(access.scopes_for(&key, at(99)), scopes);
        assert!(access.scopes_for(&key, at(100)).is_empty());
        assert_eq!(access.decide(&key, at(100)), ContextDecision::Ask);
    }

    #[test]
    fn a_refusal_wins_over_a_grant() {
        let key = sandboxed("org.example.Notes");
        let mut access = ContextAccess::default();
        access.grant(&key, ScopeSet::ALL, Expiry::Never);
        access.denied.push(key.to_string());
        assert_eq!(access.decide(&key, at(0)), ContextDecision::Denied);

        access.grant(&key, ScopeSet::ALL, Expiry::Never);
        assert_eq!(access.scopes_for(&key, at(0)), ScopeSet::ALL);
        access.deny(&key);
        assert!(access.grants.is_empty());
        assert_eq!(access.decide(&key, at(0)), ContextDecision::Denied);
    }

    #[test]
    fn scripts_never_get_raw_and_lose_their_grant_with_the_session() {
        let script = AppKey::Interpreted {
            interpreter: "/usr/bin/python3".into(),
            script: "/srv/agent.py".into(),
        };
        let mut access = ContextAccess::default();
        access.grant(&script, ScopeSet::ALL, Expiry::Never);
        assert_eq!(access.grants[&script.to_string()].expires, Expiry::Session);
        assert!(!access.scopes_for(&script, at(0)).contains(Scope::Raw));

        access.grants.insert(
            script.to_string(),
            Grant {
                scopes: vec![Scope::Raw, Scope::Text],
                expires: Expiry::Never,
            },
        );
        assert_eq!(
            access.scopes_for(&script, at(0)),
            ScopeSet::EMPTY.with(Scope::Text)
        );

        let system = executable("/usr/bin/agent");
        access.grant(&system, ScopeSet::ALL, Expiry::Session);
        access.grant(&executable("/usr/bin/other"), ScopeSet::ALL, Expiry::Never);
        assert!(access.forget_session_grants());
        assert_eq!(
            access.grants.keys().collect::<Vec<_>>(),
            ["exe:/usr/bin/other"]
        );
    }

    #[test]
    fn the_new_and_the_missing_sections_are_reachable_by_name() {
        for section in ["agents", "visual_context", "context_access"] {
            assert!(crate::schema::SECTIONS.contains(&section), "{section}");
        }
        assert!(crate::schema::check_value("context_access", "host_default", "Deny").is_ok());
        assert!(crate::schema::check_value("context_access", "host_default", "Maybe").is_err());
    }

    #[test]
    fn the_section_reads_back_what_it_wrote() -> Result<(), Box<dyn std::error::Error>> {
        let mut access = ContextAccess::default();
        access.grant(
            &sandboxed("org.example.Notes"),
            ScopeSet::EMPTY.with(Scope::Metadata),
            Expiry::At(1_900_000_000),
        );
        access.deny(&executable("/usr/bin/spy"));
        let text = ron::to_string(&access)?;
        assert_eq!(ron::from_str::<ContextAccess>(&text)?, access);
        Ok(())
    }
}
