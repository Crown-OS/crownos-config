//! Which themes and plugins are active and what each has been allowed to do.
//!
//! Written only by `crownplugind`; everything else asks it over IPC.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Granted,
    Denied,
}

crate::section! {
    pub struct Plugins in "plugins", keys PluginsKey {
        pub active_theme as ActiveTheme: Option<String> = None,
        pub enabled as Enabled: Vec<String> = Vec::new(),
        /// plugin id -> capability name -> what the user decided.
        pub grants as Grants: BTreeMap<String, BTreeMap<String, Decision>> = BTreeMap::new(),
    }
}

impl Plugins {
    pub fn decision(&self, plugin: &str, capability: &str) -> Option<Decision> {
        self.grants.get(plugin)?.get(capability).copied()
    }
}
