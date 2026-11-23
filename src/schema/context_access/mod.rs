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
mod resolve;
mod scope;
mod session;

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub use app_key::{AppKey, AppKeyError};
pub use scope::{Scope, ScopeSet};
pub use session::current_session;

/// Scopes a session-only key never gets: they capture or control other apps.
const NOT_FOR_SESSION_ONLY_KEYS: [Scope; 3] = [Scope::Raw, Scope::Engine, Scope::PolicyAdmin];

/// The one executable that may hold [`Scope::Engine`], when root controls it.
pub const CONTEXT_ENGINE_EXECUTABLE: &str = "/usr/bin/crown-contextd";

/// What happens when an application without a grant asks for context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirstUsePolicy {
    /// Refuse without a question.
    Deny,
    /// Ask the user on screen, if `ask_on_first_use` is on.
    Ask,
}

/// When a grant stops.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Expiry {
    /// The grant stays until the user removes it.
    #[default]
    Never,
    /// The grant applies only in the session with this identity, from
    /// [`current_session`]. The compositor also calls
    /// [`ContextAccess::forget_session_grants`] when a session starts.
    Session(String),
    /// The grant stops at this time, in seconds since the Unix epoch.
    At(u64),
}

impl Expiry {
    /// A session grant for the current session. Its identity is empty, and
    /// so never applies, when the session has no identity.
    pub fn this_session() -> Self {
        Self::Session(current_session().unwrap_or_default().to_owned())
    }

    pub fn is_in_force(&self, now: SystemTime, session: Option<&str>) -> bool {
        match self {
            Self::Never => true,
            Self::Session(identity) => !identity.is_empty() && session == Some(identity.as_str()),
            Self::At(seconds) => now
                .duration_since(UNIX_EPOCH)
                .is_ok_and(|elapsed| elapsed.as_secs() < *seconds),
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
        /// application gets screen pixels through the `Raw` scope. It does
        /// not apply to the `Engine` scope.
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
        self.decide_in_session(key, now, current_session())
    }

    fn decide_in_session(
        &self,
        key: &AppKey,
        now: SystemTime,
        session: Option<&str>,
    ) -> ContextDecision {
        let text = key.to_string();
        if self.denied.contains(&text) {
            return ContextDecision::Denied;
        }
        if let Some(grant) = self
            .grants
            .get(&text)
            .filter(|grant| grant.expires.is_in_force(now, session))
        {
            let stored: ScopeSet = grant.scopes.iter().collect();
            return ContextDecision::Granted(
                restrict(key, stored, is_session_only(key)).with_included(),
            );
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
    /// a grant for this session only, and never [`Scope::Raw`],
    /// [`Scope::Engine`] or [`Scope::PolicyAdmin`].
    pub fn grant(&mut self, key: &AppKey, scopes: ScopeSet, expires: Expiry) {
        if is_session_only(key) {
            self.store_grant(key, restrict(key, scopes, true), Expiry::this_session());
        } else {
            self.store_grant(key, restrict(key, scopes, false), expires);
        }
    }

    /// Stores a grant as given. [`ContextAccess::decide`] still restricts it
    /// when the key is session-only at use.
    fn store_grant(&mut self, key: &AppKey, scopes: ScopeSet, expires: Expiry) {
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

    /// Removes every session grant. Returns whether something changed.
    pub fn forget_session_grants(&mut self) -> bool {
        let before = self.grants.len();
        self.grants
            .retain(|_, grant| !matches!(grant.expires, Expiry::Session(_)));
        self.grants.len() != before
    }
}

fn is_session_only(key: &AppKey) -> bool {
    key.is_session_only(dirs::home_dir().as_deref())
}

/// The scopes `key` may hold at all: a session-only key never gets Raw,
/// Engine or PolicyAdmin, and of the executables only crown-contextd may
/// run the engine.
pub fn allowed_scopes(key: &AppKey) -> ScopeSet {
    restrict(key, ScopeSet::ALL, is_session_only(key))
}

fn restrict(key: &AppKey, scopes: ScopeSet, session_only: bool) -> ScopeSet {
    let scopes = if session_only {
        NOT_FOR_SESSION_ONLY_KEYS
            .into_iter()
            .fold(scopes, ScopeSet::without)
    } else {
        scopes
    };
    match key {
        AppKey::HostExecutable { path } if path.as_os_str() != CONTEXT_ENGINE_EXECUTABLE => {
            scopes.without(Scope::Engine)
        }
        _ => scopes,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn executable(path: &str) -> AppKey {
        AppKey::HostExecutable { path: path.into() }
    }

    fn spawned(component: &str) -> AppKey {
        AppKey::Spawned {
            component: component.into(),
        }
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
        assert_eq!(access.decide(&spawned("agent"), now), ContextDecision::Ask);
        assert_eq!(
            access.decide(&sandboxed("org.example.Notes"), now),
            ContextDecision::Denied
        );
        assert!(access.scopes_for(&spawned("agent"), now).is_empty());
    }

    #[test]
    fn without_first_use_questions_asking_becomes_a_refusal() {
        let access = ContextAccess {
            ask_on_first_use: false,
            ..ContextAccess::default()
        };
        assert_eq!(
            access.decide(&spawned("agent"), at(0)),
            ContextDecision::Denied
        );
    }

    #[test]
    fn a_grant_gives_its_scopes_until_it_expires() {
        let key = spawned("agent");
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
    fn session_only_keys_never_capture_or_control() {
        let script = AppKey::Interpreted {
            interpreter: "/usr/bin/python3".into(),
            script: "/srv/agent.py".into(),
        };
        let mut access = ContextAccess::default();
        access.grant(&script, ScopeSet::ALL, Expiry::Never);
        assert_eq!(
            access.grants[&script.to_string()].expires,
            Expiry::this_session()
        );

        access.grants.insert(
            script.to_string(),
            Grant {
                scopes: vec![Scope::Engine, Scope::PolicyAdmin, Scope::Raw, Scope::Text],
                expires: Expiry::Never,
            },
        );
        assert_eq!(
            access.scopes_for(&script, at(0)),
            ScopeSet::EMPTY.with(Scope::Text)
        );
    }

    #[test]
    fn of_the_executables_only_the_context_engine_may_run_the_engine() {
        let mut access = ContextAccess::default();
        let other = executable("/usr/bin/agent");
        access.grant(&other, ScopeSet::EMPTY.with(Scope::Engine), Expiry::Never);
        assert!(!access.scopes_for(&other, at(0)).contains(Scope::Engine));
        assert!(!allowed_scopes(&other).contains(Scope::Engine));

        let engine = executable(CONTEXT_ENGINE_EXECUTABLE);
        let root_controls_it = !engine.is_session_only(dirs::home_dir().as_deref());
        assert_eq!(
            allowed_scopes(&engine).contains(Scope::Engine),
            root_controls_it
        );
    }

    #[test]
    fn the_engine_scope_includes_what_it_observes() {
        let key = spawned("agent");
        let mut access = ContextAccess::default();
        access.grant(&key, ScopeSet::EMPTY.with(Scope::Engine), Expiry::Never);
        let scopes = access.scopes_for(&key, at(0));
        for scope in [
            Scope::Engine,
            Scope::Metadata,
            Scope::Text,
            Scope::Raw,
            Scope::Pause,
        ] {
            assert!(scopes.contains(scope), "{scope}");
        }
    }

    #[test]
    fn a_session_grant_applies_only_in_its_own_session() {
        let key = spawned("agent");
        let mut access = ContextAccess::default();
        access.grant(&key, ScopeSet::ALL, Expiry::Session("boot-a".to_owned()));
        let granted = |session| access.decide_in_session(&key, at(0), session);
        assert_eq!(
            granted(Some("boot-a")),
            ContextDecision::Granted(ScopeSet::ALL)
        );
        assert_eq!(granted(Some("boot-b")), ContextDecision::Ask);
        assert_eq!(granted(None), ContextDecision::Ask);

        access.grant(&key, ScopeSet::ALL, Expiry::Session(String::new()));
        assert_eq!(
            access.decide_in_session(&key, at(0), Some("")),
            ContextDecision::Ask
        );
    }

    #[test]
    fn forgetting_session_grants_keeps_the_lasting_ones() {
        let mut access = ContextAccess::default();
        access.grant(&spawned("agent"), ScopeSet::ALL, Expiry::this_session());
        access.grant(&spawned("lasting"), ScopeSet::ALL, Expiry::Never);
        access.grant(
            &AppKey::Interpreted {
                interpreter: "/usr/bin/python3".into(),
                script: "/srv/agent.py".into(),
            },
            ScopeSet::ALL,
            Expiry::Never,
        );
        assert!(access.forget_session_grants());
        assert_eq!(
            access.grants.keys().collect::<Vec<_>>(),
            ["spawned:lasting"]
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
