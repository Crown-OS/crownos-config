//! AI agents on this desktop: which programs may drive it, and which
//! applications may show their UI to agents.
//!
//! The consumer is [crownpositor], which reads this file live. It gates the
//! agent protocols (window control, agent cursors, the context stream) by the
//! executable behind a client, and answers `crownos_agent_access_v1` requests
//! from the remembered decisions here, asking on screen for the rest.
//!
//! [crownpositor]: https://github.com/crown-os/crownpositor

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A remembered answer to an application asking to expose its UI to agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    Allow,
    Deny,
}

crate::section! {
    pub struct Agents in "agents", keys AgentsKey {
        /// Programs, by absolute path, that may bind the agent protocols and
        /// so see every window, move them and drive them through their own
        /// cursors. The compositor checks the executable behind each client
        /// when it connects.
        pub trusted_executables as TrustedExecutables: Vec<String> = default_trusted_executables(),

        /// Ask on screen when an application asks to expose its UI to agents
        /// and no answer is remembered below. Off, such requests are denied.
        pub ask_for_app_access as AskForAppAccess: bool = true,

        /// Remembered answers per application id.
        pub app_access as AppAccess: BTreeMap<String, AccessDecision> = BTreeMap::new(),
    }
}

fn default_trusted_executables() -> Vec<String> {
    [
        "/usr/bin/crownos-computer-use",
        "/usr/bin/crown-contextd",
        "/usr/bin/crownlink-linux",
        "/usr/bin/crownos-remoted",
    ]
    .map(str::to_owned)
    .to_vec()
}
