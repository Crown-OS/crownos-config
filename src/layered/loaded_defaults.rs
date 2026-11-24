//! The defaults that each section had when this process last read it. A save
//! compares with these and not with the files on disk: when an update replaces
//! the distro defaults while an app runs, the app's value still comes from the
//! old defaults, and a comparison with the new ones would pin each old default
//! in the user's file.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};

use crate::section_keys::SectionKeys;

static LOADED_DEFAULTS: LazyLock<Mutex<HashMap<String, SectionKeys>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn lock() -> MutexGuard<'static, HashMap<String, SectionKeys>> {
    LOADED_DEFAULTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn remember(section: &str, defaults: SectionKeys) {
    lock().insert(section.to_owned(), defaults);
}

pub(super) fn recall(section: &str) -> Option<SectionKeys> {
    lock().get(section).cloned()
}
