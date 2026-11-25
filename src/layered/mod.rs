//! Reads a section as its default layers with the user's file on top, and
//! writes only the keys that the user changed, with the other keys as
//! commented defaults. See [`crate::layers`].

mod loaded_defaults;
mod migrate;
mod read;
mod user_file;
mod validate;
mod write;

pub use read::merged_section_text;
pub(crate) use read::read_section;
pub use user_file::USER_FILE_HEADER;
pub use validate::validate_default_layer;
pub use write::reset_keys;
pub(crate) use write::{write_section, write_section_edit};
