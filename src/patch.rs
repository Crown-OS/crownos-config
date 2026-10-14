//! Field-level edits addressed by name, for writers that only know a section
//! and a key at runtime — a theme manifest saying `appearance.blur_passes = 4`.
//!
//! Values travel as RON text but are always parsed into the field's real type,
//! so enum variants such as `AccentColor::Blue` survive a round trip that a
//! generic `ron::Value` would flatten.

use std::io;

use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::parser::options;
use crate::util::path_for;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("unknown section `{0}`")]
    UnknownSection(String),
    #[error("section `{section}` has no key `{key}`")]
    UnknownKey { section: &'static str, key: String },
    #[error("`{section}.{key}`: {message}")]
    InvalidValue {
        section: &'static str,
        key: String,
        message: String,
    },
    #[error("`{section}.ron` does not parse: {message}")]
    Unparseable { section: String, message: String },
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// A settings struct whose fields can be read and written by name.
///
/// Implemented for every section by [`section!`](crate::section).
pub trait DynSection {
    fn section_name(&self) -> &'static str;
    fn key_names(&self) -> &'static [&'static str];
    fn get_ron(&self, key: &str) -> Result<String, PatchError>;
    fn set_ron(&mut self, key: &str, text: &str) -> Result<(), PatchError>;
}

#[doc(hidden)]
pub fn to_ron<T: Serialize>(
    section: &'static str,
    key: &str,
    value: &T,
) -> Result<String, PatchError> {
    ron::to_string(value).map_err(|error| PatchError::InvalidValue {
        section,
        key: key.to_owned(),
        message: error.to_string(),
    })
}

#[doc(hidden)]
pub fn from_ron<T: DeserializeOwned>(
    section: &'static str,
    key: &str,
    text: &str,
) -> Result<T, PatchError> {
    options()
        .from_str(text)
        .map_err(|error| PatchError::InvalidValue {
            section,
            key: key.to_owned(),
            message: error.to_string(),
        })
}

/// Read a section, treating a missing file as the default but refusing one
/// that exists and does not parse — a writer must never replace a file the
/// user is halfway through editing with defaults.
pub fn load_strict<T: DeserializeOwned + Default>(section: &str) -> Result<T, PatchError> {
    match std::fs::read_to_string(path_for(section)) {
        Ok(text) => options()
            .from_str(&text)
            .map_err(|error| PatchError::Unparseable {
                section: section.to_owned(),
                message: error.to_string(),
            }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn edit_typed<T, R>(
    section: &str,
    edit: impl FnOnce(&mut dyn DynSection) -> Result<R, PatchError>,
) -> Result<R, PatchError>
where
    T: DynSection + DeserializeOwned + Serialize + Default + PartialEq + Clone,
{
    let original: T = load_strict(section)?;
    let mut edited = original.clone();
    let result = edit(&mut edited)?;
    if edited != original {
        crate::save(section, &edited)?;
    }
    Ok(result)
}

pub(crate) fn read_typed<T, R>(
    section: &str,
    read: impl FnOnce(&dyn DynSection) -> Result<R, PatchError>,
) -> Result<R, PatchError>
where
    T: DynSection + DeserializeOwned + Default,
{
    read(&load_strict::<T>(section)?)
}

/// The RON text of one key as it currently is on disk.
pub fn read_key(section: &str, key: &str) -> Result<String, PatchError> {
    crate::schema::read_section(section, |fields| fields.get_ron(key))
}

/// Apply several `(key, ron)` edits to one section and write it once.
/// Nothing is written when any edit fails or nothing changes.
pub fn write_keys<'a>(
    section: &str,
    edits: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<(), PatchError> {
    crate::schema::edit_section(section, |fields| {
        edits
            .into_iter()
            .try_for_each(|(key, text)| fields.set_ron(key, text))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AccentColor, Appearance};

    #[test]
    fn enum_values_survive_a_round_trip() {
        let mut appearance = Appearance::default();
        appearance.set_ron("accent", "Blue").expect("valid variant");
        assert_eq!(appearance.accent, AccentColor::Blue);
        assert_eq!(appearance.get_ron("accent").expect("known key"), "Blue");
    }

    #[test]
    fn a_value_of_the_wrong_type_is_refused() {
        let mut appearance = Appearance::default();
        assert!(matches!(
            appearance.set_ron("blur_passes", "\"four\""),
            Err(PatchError::InvalidValue { .. })
        ));
        assert_eq!(appearance, Appearance::default());
    }

    #[test]
    fn unknown_keys_and_sections_are_reported() {
        let mut appearance = Appearance::default();
        assert!(matches!(
            appearance.set_ron("nope", "1"),
            Err(PatchError::UnknownKey { .. })
        ));
        assert!(matches!(
            crate::schema::read_section("nope", |_| Ok(())),
            Err(PatchError::UnknownSection(_))
        ));
    }

    #[test]
    fn optional_fields_accept_implicit_some() {
        let mut bar = crate::schema::Bar::default();
        bar.set_ron("height", "40").expect("implicit Some");
        assert_eq!(bar.height, Some(40));
    }
}
