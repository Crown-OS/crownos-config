use std::io;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::read::read_default_layers;
use super::{loaded_defaults, migrate, user_file};
use crate::layers::SectionFiles;
use crate::section_keys::SectionKeys;
use crate::util::{hash_bytes, record_written};

/// Writes the keys of `value` that the user changed: the keys that the user's
/// file already has, and the keys whose value differs from the defaults that
/// the last read of the section gave.
pub(crate) fn write_section<T: Serialize + DeserializeOwned + Default>(
    section: &str,
    files: &SectionFiles,
    value: &T,
) -> io::Result<()> {
    let mut user_changes = SectionKeys::of_value(value).map_err(invalid_data)?;
    let defaults =
        loaded_defaults::recall(section).unwrap_or_else(|| current_canonical_defaults::<T>(files));
    let existing_user_keys = existing_user_changes::<T>(section, files, &defaults);
    user_changes
        .retain(|key, value| existing_user_keys.contains(key) || defaults.get(key) != Some(value));
    write_user_file(section, files, &user_changes, &defaults)
}

/// Like [`write_section`] for an edit of single keys, such as a theme that
/// sets a key and later sets it back: a key that the edit sets to its default
/// leaves the user's file, so the file is the same as before the theme.
pub(crate) fn write_section_edit<T: Serialize + DeserializeOwned + Default>(
    section: &str,
    files: &SectionFiles,
    original: &T,
    edited: &T,
) -> io::Result<()> {
    let original_keys = SectionKeys::of_value(original).map_err(invalid_data)?;
    let mut user_changes = SectionKeys::of_value(edited).map_err(invalid_data)?;
    let defaults =
        loaded_defaults::recall(section).unwrap_or_else(|| current_canonical_defaults::<T>(files));
    let existing_user_keys = existing_user_changes::<T>(section, files, &defaults);
    user_changes.retain(|key, value| {
        let is_default = defaults.get(key) == Some(value);
        let edited_now = original_keys.get(key) != Some(value);
        !is_default || (existing_user_keys.contains(key) && !edited_now)
    });
    write_user_file(section, files, &user_changes, &defaults)
}

/// Removes `keys` from the user's file, so that they follow the defaults again.
/// The comments show the defaults of the last read of the section in this
/// process, or else the default layers; the next read of the section writes
/// the comments of the keys that only the program knows.
pub fn reset_keys(section: &str, keys: &[&str]) -> io::Result<()> {
    let files = SectionFiles::for_section(section);
    let text = match std::fs::read_to_string(&files.user) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let mut user_changes = SectionKeys::parse(&text).map_err(invalid_data)?;
    user_changes.retain(|key, _| !keys.contains(&key));
    let defaults = loaded_defaults::recall(section).unwrap_or_else(|| read_default_layers(&files));
    write_user_file(section, &files, &user_changes, &defaults)
}

/// Writes the file atomically: a reader or a watcher never sees half of it.
/// The merged text is recorded first, because the watcher can see the rename
/// as soon as it is done.
pub(super) fn write_user_file(
    section: &str,
    files: &SectionFiles,
    user_changes: &SectionKeys,
    defaults: &SectionKeys,
) -> io::Result<()> {
    let merged = read_default_layers(files).overlaid(user_changes.clone());
    record_written(section, hash_bytes(merged.to_ron().as_bytes()));

    if let Some(directory) = files.user.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let text = user_file::render(section, defaults, user_changes);
    let temporary = files
        .user
        .with_extension(format!("ron.tmp.{}", std::process::id()));
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, &files.user)
}

fn existing_user_changes<T: Serialize + DeserializeOwned + Default>(
    section: &str,
    files: &SectionFiles,
    canonical_defaults: &SectionKeys,
) -> SectionKeys {
    let Ok(text) = std::fs::read_to_string(&files.user) else {
        return SectionKeys::default();
    };
    let Ok(user_keys) = SectionKeys::parse(&text) else {
        return SectionKeys::default();
    };
    if migrate::keeps_user_file(files) {
        return user_keys;
    }
    migrate::reduce_to_user_changes::<T>(
        section,
        files,
        &read_default_layers(files),
        canonical_defaults,
        user_keys,
    )
}

fn current_canonical_defaults<T: Serialize + DeserializeOwned + Default>(
    files: &SectionFiles,
) -> SectionKeys {
    let defaults: T = read_default_layers(files).parse_value().unwrap_or_default();
    SectionKeys::of_value(&defaults).unwrap_or_default()
}

fn invalid_data(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
