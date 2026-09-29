//! The clipboard manager: how much of what was copied outlives the app that
//! copied it.
//!
//! The consumer is [crownos-clipboard], which reads this file at startup.
//!
//! [crownos-clipboard]: https://github.com/crown-os/crownos-clipboard

crate::section! {
    pub struct Clipboard in "clipboard", keys ClipboardKey {
        /// How many past copies to remember. Zero keeps only the current one.
        pub history_size as HistorySize: u32 = 50,

        /// The largest copy kept, in MiB, counting every format it was offered
        /// in. Anything bigger still pastes while its app is running but is
        /// neither remembered nor kept alive after it exits.
        pub max_entry_mib as MaxEntryMib: u32 = 16,

        /// Also keep the primary selection (middle-click paste) alive after
        /// its app exits. Off by default: it changes on every text highlight.
        pub persist_primary as PersistPrimary: bool = false,
    }
}
