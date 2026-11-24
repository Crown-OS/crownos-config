//! Reads a section as its default layers with the user's file on top, and
//! writes only the keys that the user changed. See [`crate::layers`].

mod loaded_defaults;
mod migrate;
mod read;
mod validate;
mod write;

pub use read::merged_section_text;
pub(crate) use read::read_section;
pub use validate::validate_default_layer;
pub use write::reset_keys;
pub(crate) use write::write_section;

/// The first line of each user file that this crate writes. A user file
/// without it was copied or written out in full by an earlier release, and is
/// reduced to the user's own changes the first time it is read.
pub const USER_FILE_HEADER: &str = "// Only the settings that you changed. The CrownOS defaults are in /usr/share/crownos/defaults.";

fn has_user_file_header(text: &str) -> bool {
    text.lines().next() == Some(USER_FILE_HEADER)
}
