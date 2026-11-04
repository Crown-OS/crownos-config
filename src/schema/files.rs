//! The file manager: how folders are listed and what happens when a drive is
//! plugged in.
//!
//! The consumer is crownos-files, which follows this file while it runs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortKey {
    #[default]
    Name,
    Modified,
    Created,
    Size,
    Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagColor {
    pub name: String,
    /// `0xRRGGBB`.
    pub rgb: u32,
}

fn default_tags() -> Vec<TagColor> {
    [
        ("Red", 0xe5484d),
        ("Orange", 0xd97706),
        ("Yellow", 0xfbbf24),
        ("Green", 0x4ade80),
        ("Blue", 0x3b82f6),
        ("Purple", 0x8f6dfb),
        ("Gray", 0xa1a1aa),
    ]
    .into_iter()
    .map(|(name, rgb)| TagColor {
        name: name.to_owned(),
        rgb,
    })
    .collect()
}

crate::section! {
    pub struct Files in "files", keys FilesKey {
        /// List dotfiles and other hidden entries.
        pub show_hidden as ShowHidden: bool = false,

        /// What folders are ordered by.
        pub sort_by as SortBy: SortKey = SortKey::Name,

        /// Reverse the order `sort_by` gives.
        pub sort_reverse as SortReverse: bool = false,

        /// Keep folders above files whatever the sort key.
        pub dirs_first as DirsFirst: bool = true,

        /// Icons or rows.
        pub view_mode as View: ViewMode = ViewMode::Grid,

        /// Mount removable drives as soon as they are plugged in.
        pub automount as Automount: bool = true,

        /// Open a window on a drive after mounting it automatically.
        pub automount_open as AutomountOpen: bool = false,

        /// Ask before deleting without the trash.
        pub confirm_permanent_delete as ConfirmPermanentDelete: bool = true,

        /// The tags offered in the tag menu and the color each is drawn in.
        pub tags as Tags: Vec<TagColor> = default_tags(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_gives_the_defaults() {
        let files: Files = ron::from_str("()").unwrap_or_default();
        assert_eq!(files, Files::default());
        assert!(files.automount);
        assert_eq!(files.tags.len(), 7);
    }

    #[test]
    fn a_partial_file_keeps_the_other_defaults() {
        let files: Files =
            ron::from_str("(show_hidden: true, view_mode: List)").unwrap_or_default();
        assert!(files.show_hidden);
        assert_eq!(files.view_mode, ViewMode::List);
        assert!(files.dirs_first);
    }
}
