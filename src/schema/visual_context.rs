//! The visual context engine: what it may look at and how long it remembers.
//!
//! Two consumers: [crownpositor] reads the exclusion rules live and enforces
//! them in the compositor, so a window stays out of captures even when the
//! engine is not running; [crown-contextd] reads everything else.
//!
//! [crownpositor]: https://github.com/crown-os/crownpositor
//! [crown-contextd]: https://github.com/crown-os/crownos-visual-context-engine

crate::section! {
    pub struct VisualContext in "visual_context", keys VisualContextKey {
        /// Whether the engine observes at all.
        pub enabled as Enabled: bool = false,

        /// Application ids, as globs, whose windows are never captured or
        /// described: `org.keepassxc.*`, `*bank*`.
        pub excluded_apps as ExcludedApps: Vec<String> = default_excluded_apps(),

        /// Window titles, as regular expressions, that mark a private
        /// browsing window. Matching windows are never captured.
        pub private_titles as PrivateTitles: Vec<String> = default_private_titles(),

        /// The keybinding that pauses and resumes observation.
        pub pause_hotkey as PauseHotkey: String = "Super+Shift+period".to_owned(),

        /// Days raw text and thumbnails are kept before pruning.
        pub retention_days as RetentionDays: u32 = 30,
    }
}

fn default_excluded_apps() -> Vec<String> {
    ["org.keepassxc.KeePassXC", "org.gnome.seahorse.Application", "1password", "bitwarden"]
        .map(str::to_owned)
        .to_vec()
}

fn default_private_titles() -> Vec<String> {
    [
        r"(?i)private browsing",
        r"(?i)\(private\)$",
        r"(?i)incognito",
        r"(?i)inprivate",
        r"(?i)navigation privée",
        r"(?i)privates fenster",
        r"(?i)navegación privada",
        r"(?i)プライベート",
    ]
    .map(str::to_owned)
    .to_vec()
}
