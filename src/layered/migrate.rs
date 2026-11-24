//! Earlier releases copied the distro files into each new home and wrote out
//! every field the first time an app ran. Such a file pins each default for
//! ever. The first read reduces it to the keys whose value differs from the
//! current defaults and from the defaults of each earlier release. The file
//! as it was stays beside it as `<section>.ron.before-layers`: a rollback to a
//! release without layers reads only the user's file, and the user can copy
//! it back.

use std::{io, iter};

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::write::write_user_file;
use crate::layers::SectionFiles;
use crate::section_keys::SectionKeys;

const BEFORE_LAYERS_EXTENSION: &str = "ron.before-layers";

/// The user's changes after the reduction, or `user_keys` unchanged when the
/// reduction cannot parse the section or cannot write the result.
pub(super) fn reduce_to_user_changes<T: DeserializeOwned + Serialize + Default>(
    section: &str,
    files: &SectionFiles,
    default_keys: &SectionKeys,
    canonical_defaults: &SectionKeys,
    user_keys: SectionKeys,
) -> SectionKeys {
    let merged = default_keys.clone().overlaid(user_keys.clone());
    let Some(mut user_changes) = merged
        .parse_value::<T>()
        .ok()
        .and_then(|value| SectionKeys::of_value(&value).ok())
    else {
        return user_keys;
    };
    let shipped_defaults: Vec<SectionKeys> = iter::once(canonical_defaults.clone())
        .chain(earlier_release_defaults::<T>(files))
        .collect();
    user_changes.retain(|key, value| {
        user_keys.contains(key)
            && !shipped_defaults
                .iter()
                .any(|defaults| defaults.get(key) == Some(value))
    });
    let written = keep_file_before_layers(files)
        .and_then(|()| write_user_file(section, files, &user_changes));
    match written {
        Ok(()) => user_changes,
        Err(_) => user_keys,
    }
}

fn keep_file_before_layers(files: &SectionFiles) -> io::Result<()> {
    let copy = files.user.with_extension(BEFORE_LAYERS_EXTENSION);
    if copy.exists() {
        return Ok(());
    }
    std::fs::copy(&files.user, copy).map(drop)
}

fn earlier_release_defaults<T: DeserializeOwned + Serialize + Default>(
    files: &SectionFiles,
) -> impl Iterator<Item = SectionKeys> {
    files
        .earlier_release_defaults
        .iter()
        .filter_map(|file| std::fs::read_to_string(file).ok())
        .filter_map(|text| SectionKeys::parse(&text).ok()?.parse_value::<T>().ok())
        .filter_map(|defaults| SectionKeys::of_value(&defaults).ok())
}
