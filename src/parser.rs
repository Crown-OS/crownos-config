use std::io;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::layered::{read_section, write_section};
use crate::layers::SectionFiles;

/// `Option` fields are read as `floating: true` rather than `Some(true)`, so an
/// optional setting is written the same way a required one is. Explicit
/// `Some(..)` — which is what [`save`] emits — still parses.
pub(crate) fn options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

/// Read a section: the default layers with the user's file on top.
///
/// Nothing is written for a missing file, so each key that the user did not
/// change follows the defaults of the installed release. A user file that does
/// not parse is ignored for this read and the defaults are returned: replacing
/// a file that the user is halfway through editing would be worse.
pub fn load<T: DeserializeOwned + Serialize + Default>(section: &str) -> T {
    read_section(section, &SectionFiles::for_section(section))
        .unwrap_or_else(|fallback| fallback.defaults)
}

/// Write the keys of a section that the user changed, atomically.
///
/// A key is written when the user's file already has it, or when its value
/// differs from the defaults that the last [`load`] of the section gave. The
/// merged text is recorded for [echo suppression](crate#echo-suppression), so
/// subscribers of this section do not get notified about this app's own write.
pub fn save<T: Serialize + DeserializeOwned + Default>(section: &str, value: &T) -> io::Result<()> {
    write_section(section, &SectionFiles::for_section(section), value)
}
