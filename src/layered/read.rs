use std::io;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::write::write_user_file;
use super::{loaded_defaults, migrate, user_file};
use crate::layers::SectionFiles;
use crate::patch::PatchError;
use crate::section_keys::SectionKeys;

/// The section without the user's file, for a reader that must not stop when
/// that file cannot be read.
pub(crate) struct Fallback<T> {
    pub defaults: T,
    pub error: PatchError,
}

/// The default layers with the user's file on top. Each read also records the
/// defaults for the next [`crate::save`] of the section, and writes the user's
/// file when it is missing or a program wrote it for defaults that changed
/// since. A failed
/// write changes nothing that the read gives.
pub(crate) fn read_section<T: DeserializeOwned + Serialize + Default>(
    section: &str,
    files: &SectionFiles,
) -> Result<T, Fallback<T>> {
    let default_keys = read_default_layers(files);
    let defaults: T = default_keys.parse_value().unwrap_or_default();
    let canonical_defaults = SectionKeys::of_value(&defaults).unwrap_or_default();
    loaded_defaults::remember(section, canonical_defaults.clone());

    let user_text = match std::fs::read_to_string(&files.user) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            migrate::mark_reduced(files);
            if !canonical_defaults.is_empty() {
                let _ =
                    write_user_file(section, files, &SectionKeys::default(), &canonical_defaults);
            }
            return Ok(defaults);
        }
        Err(error) => {
            return Err(Fallback {
                defaults,
                error: error.into(),
            });
        }
    };
    let user_keys = match SectionKeys::parse(&user_text) {
        Ok(keys) => keys,
        Err(error) => {
            return Err(Fallback {
                defaults,
                error: unparseable(section, &error),
            });
        }
    };
    let user_keys = if migrate::keeps_user_file(files) {
        if user_file::has_outdated_defaults(&user_text, &canonical_defaults) {
            let _ = write_user_file(section, files, &user_keys, &canonical_defaults);
        }
        user_keys
    } else {
        migrate::reduce_to_user_changes::<T>(
            section,
            files,
            &default_keys,
            &canonical_defaults,
            user_keys,
        )
    };
    match default_keys.overlaid(user_keys).parse_value() {
        Ok(value) => Ok(value),
        Err(error) => Err(Fallback {
            defaults,
            error: unparseable(section, &error),
        }),
    }
}

/// The RON text of the default layers with the user's file on top, before any
/// field default applies. `None` when the user's file cannot be read or parsed.
pub fn merged_section_text(section: &str) -> Option<String> {
    merged_text(&SectionFiles::for_section(section))
}

pub(crate) fn merged_text(files: &SectionFiles) -> Option<String> {
    let user_keys = match std::fs::read_to_string(&files.user) {
        Ok(text) => SectionKeys::parse(&text).ok()?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => SectionKeys::default(),
        Err(_) => return None,
    };
    Some(read_default_layers(files).overlaid(user_keys).to_ron())
}

/// A layer file that is missing or does not parse adds nothing.
pub(super) fn read_default_layers(files: &SectionFiles) -> SectionKeys {
    files
        .default_layers
        .iter()
        .filter_map(|layer| std::fs::read_to_string(layer).ok())
        .filter_map(|text| SectionKeys::parse(&text).ok())
        .fold(SectionKeys::default(), SectionKeys::overlaid)
}

fn unparseable(section: &str, error: &ron::error::SpannedError) -> PatchError {
    PatchError::Unparseable {
        section: section.to_owned(),
        message: error.to_string(),
    }
}
