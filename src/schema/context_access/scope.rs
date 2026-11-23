use std::fmt;

use serde::{Deserialize, Serialize};

/// One thing an application may do with screen context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Scope {
    /// Window list, app ids, focus, placement and activity counts. No titles.
    Metadata,
    /// Window titles, selection types and recognized text.
    Text,
    /// Raw text: unredacted search results, snippets and exports. No pixels are ever stored.
    /// The user authenticates again after `raw_reauth_minutes`.
    Raw,
    /// Add to the context memory.
    MemoryWrite,
    /// Delete from the context memory.
    MemoryDelete,
    /// Change exclusions, retention and the grants of other apps.
    PolicyAdmin,
    /// Pause and resume observation, and nothing more.
    Pause,
    /// Run the context engine: pause every stream and capture. Includes
    /// `Metadata`, `Text`, `Raw` and `Pause`.
    Engine,
    /// Move, focus and close the windows of other apps.
    WaylandWindowControl,
    /// Drive an agent cursor.
    WaylandAgentCursor,
}

impl Scope {
    pub const ALL: [Self; 10] = [
        Self::Metadata,
        Self::Text,
        Self::Raw,
        Self::MemoryWrite,
        Self::MemoryDelete,
        Self::PolicyAdmin,
        Self::Pause,
        Self::Engine,
        Self::WaylandWindowControl,
        Self::WaylandAgentCursor,
    ];

    /// The stable name for logs, audit records and user interfaces.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Text => "text",
            Self::Raw => "raw",
            Self::MemoryWrite => "memory_write",
            Self::MemoryDelete => "memory_delete",
            Self::PolicyAdmin => "policy_admin",
            Self::Pause => "pause",
            Self::Engine => "engine",
            Self::WaylandWindowControl => "wayland.window_control",
            Self::WaylandAgentCursor => "wayland.agent_cursor",
        }
    }

    const fn bit(self) -> u16 {
        1 << self as u16
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A set of [`Scope`]s, small enough to copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ScopeSet(u16);

impl ScopeSet {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self((1 << Scope::ALL.len()) - 1);

    pub const fn contains(self, scope: Scope) -> bool {
        self.0 & scope.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn with(self, scope: Scope) -> Self {
        Self(self.0 | scope.bit())
    }

    pub const fn without(self, scope: Scope) -> Self {
        Self(self.0 & !scope.bit())
    }

    /// The set with the scopes that [`Scope::Engine`] includes.
    pub const fn with_included(self) -> Self {
        if self.contains(Scope::Engine) {
            self.with(Scope::Metadata)
                .with(Scope::Text)
                .with(Scope::Raw)
                .with(Scope::Pause)
        } else {
            self
        }
    }

    pub fn iter(self) -> impl Iterator<Item = Scope> {
        Scope::ALL
            .into_iter()
            .filter(move |scope| self.contains(*scope))
    }
}

impl FromIterator<Scope> for ScopeSet {
    fn from_iter<I: IntoIterator<Item = Scope>>(scopes: I) -> Self {
        scopes.into_iter().fold(Self::EMPTY, Self::with)
    }
}

impl<'a> FromIterator<&'a Scope> for ScopeSet {
    fn from_iter<I: IntoIterator<Item = &'a Scope>>(scopes: I) -> Self {
        scopes.into_iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scope_has_its_own_bit() {
        let all: ScopeSet = Scope::ALL.iter().collect();
        assert_eq!(all, ScopeSet::ALL);
        assert_eq!(ScopeSet::ALL.iter().count(), Scope::ALL.len());
    }

    #[test]
    fn the_engine_includes_what_it_must_observe() {
        let engine = ScopeSet::EMPTY.with(Scope::Engine).with_included();
        for scope in [Scope::Metadata, Scope::Text, Scope::Raw, Scope::Pause] {
            assert!(engine.contains(scope), "{scope}");
        }
        assert!(!engine.contains(Scope::PolicyAdmin));
        let text = ScopeSet::EMPTY.with(Scope::Text);
        assert_eq!(text.with_included(), text);
    }

    #[test]
    fn with_and_without_change_one_scope() {
        let set = ScopeSet::EMPTY.with(Scope::Text).with(Scope::Raw);
        assert!(set.contains(Scope::Text));
        assert!(!set.without(Scope::Raw).contains(Scope::Raw));
        assert_eq!(set.iter().collect::<Vec<_>>(), [Scope::Text, Scope::Raw]);
    }
}
