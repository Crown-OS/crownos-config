//! A section file as its top-level keys, each with the RON text of its value.
//! The text is kept as written, so a layer can replace single keys of a lower
//! layer without parsing the values into a type that would lose enum variants.
//! Most sections are structs, `(key: value)`. A section that is a map, such as
//! the settings of a plugin, is written as a map again, `{"key": value}`.

use std::fmt;

use ron::value::RawValue;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::parser::options;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum WrittenForm {
    #[default]
    Struct,
    Map,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SectionKeys {
    entries: Vec<(String, String)>,
    form: WrittenForm,
}

impl SectionKeys {
    pub fn parse(text: &str) -> Result<Self, ron::error::SpannedError> {
        let keys: Self = options().from_str(text)?;
        let form = if starts_as_map(text) {
            WrittenForm::Map
        } else {
            WrittenForm::Struct
        };
        Ok(Self { form, ..keys })
    }

    /// The keys of `value` as [`crate::save`] writes them. Two values give the
    /// same text for a key only when that field is equal.
    pub fn of_value<T: Serialize>(value: &T) -> Result<Self, ron::Error> {
        let text = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())?;
        Self::parse(&text).map_err(|error| error.code)
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Each key of `higher` replaces the same key here, or is added at the end.
    pub fn overlay(&mut self, higher: Self) {
        if self.entries.is_empty() {
            self.form = higher.form;
        }
        for (name, value) in higher.entries {
            match self
                .entries
                .iter_mut()
                .find(|(existing, _)| *existing == name)
            {
                Some(entry) => entry.1 = value,
                None => self.entries.push((name, value)),
            }
        }
    }

    pub fn overlaid(mut self, higher: Self) -> Self {
        self.overlay(higher);
        self
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&str, &str) -> bool) {
        self.entries.retain(|(name, value)| keep(name, value));
    }

    pub fn parse_value<T: serde::de::DeserializeOwned>(
        &self,
    ) -> Result<T, ron::error::SpannedError> {
        options().from_str(&self.to_ron())
    }

    pub fn to_ron(&self) -> String {
        let (open, close) = match self.form {
            WrittenForm::Struct => ('(', ')'),
            WrittenForm::Map => ('{', '}'),
        };
        let mut text = format!("{open}\n");
        for (name, value) in &self.entries {
            text.push_str("    ");
            match self.form {
                WrittenForm::Struct => text.push_str(name),
                WrittenForm::Map => text.push_str(&quoted(name)),
            }
            text.push_str(": ");
            text.push_str(value);
            text.push_str(",\n");
        }
        text.push(close);
        text.push('\n');
        text
    }
}

fn quoted(name: &str) -> String {
    ron::to_string(name).unwrap_or_else(|_| format!("{name:?}"))
}

/// True when the first token after comments and `#![enable(..)]` attributes
/// opens a map.
fn starts_as_map(text: &str) -> bool {
    let mut rest = text;
    loop {
        rest = rest.trim_start();
        if let Some(comment) = rest.strip_prefix("//") {
            rest = comment.split_once('\n').map_or("", |(_, after)| after);
        } else if let Some(comment) = rest.strip_prefix("/*") {
            rest = comment.split_once("*/").map_or("", |(_, after)| after);
        } else if let Some(attribute) = rest.strip_prefix("#!") {
            rest = attribute.split_once(']').map_or("", |(_, after)| after);
        } else {
            return rest.starts_with('{');
        }
    }
}

impl<'de> Deserialize<'de> for SectionKeys {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(SectionKeysVisitor)
    }
}

struct SectionKeysVisitor;

impl<'de> Visitor<'de> for SectionKeysVisitor {
    type Value = SectionKeys;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a RON struct with named fields")
    }

    fn visit_unit<E: de::Error>(self) -> Result<SectionKeys, E> {
        Ok(SectionKeys::default())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<SectionKeys, A::Error> {
        let mut keys = SectionKeys::default();
        while let Some((name, value)) = access.next_entry::<String, Box<RawValue>>()? {
            keys.overlay(SectionKeys {
                entries: vec![(name, value.trim().get_ron().to_owned())],
                form: WrittenForm::Struct,
            });
        }
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
    enum Accent {
        #[default]
        Purple,
        Blue,
    }

    #[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
    enum Mode {
        #[default]
        Off,
        On {
            level: u8,
        },
        Pair(u8, u8),
    }

    #[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
    #[serde(default)]
    struct Inner {
        first: u32,
        second: Option<u32>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(default)]
    struct Section {
        accent: Accent,
        height: Option<u32>,
        items: Vec<String>,
        inner: Inner,
        mode: Mode,
    }

    impl Default for Section {
        fn default() -> Self {
            Self {
                accent: Accent::Purple,
                height: Some(30),
                items: vec!["distro".into()],
                inner: Inner::default(),
                mode: Mode::Off,
            }
        }
    }

    fn keys(text: &str) -> SectionKeys {
        SectionKeys::parse(text).unwrap_or_default()
    }

    #[test]
    fn struct_syntax_gives_its_keys_with_comments_and_trailing_commas() {
        let parsed = keys("// a comment\n(\n    accent: Blue,\n    items: [\"a\"],\n)");
        assert_eq!(parsed.get("accent"), Some("Blue"));
        assert_eq!(parsed.get("items"), Some("[\"a\"]"));
    }

    #[test]
    fn named_unit_and_implicit_some_files_parse() {
        assert_eq!(keys("Section(accent: Blue)").get("accent"), Some("Blue"));
        assert_eq!(keys("()"), SectionKeys::default());
        assert_eq!(
            keys("#![enable(implicit_some)]\n(height: 5)").get("height"),
            Some("5")
        );
    }

    #[test]
    fn a_map_section_is_written_as_a_map_again() {
        use std::collections::BTreeMap;

        let settings: BTreeMap<String, u32> =
            [("volume level".to_owned(), 3), ("speed".to_owned(), 7)].into();
        let canonical = SectionKeys::of_value(&settings).unwrap_or_default();
        assert_eq!(canonical.get("volume level"), Some("3"));
        let written = canonical.to_ron();
        assert!(written.starts_with('{'), "{written}");
        let read_back: BTreeMap<String, u32> = canonical.parse_value().unwrap_or_default();
        assert_eq!(read_back, settings);

        let merged = SectionKeys::default().overlaid(keys("// note\n{\"speed\": 9}"));
        let merged: BTreeMap<String, u32> = merged.parse_value().unwrap_or_default();
        assert_eq!(merged.get("speed"), Some(&9));
    }

    #[test]
    fn the_form_is_found_after_comments_and_attributes() {
        assert!(starts_as_map(
            "/* a */ // b\n#![enable(implicit_some)]\n {}"
        ));
        assert!(!starts_as_map("// {\n(a: 1)"));
        assert!(!starts_as_map("Section(a: 1)"));
    }

    #[test]
    fn a_file_that_is_not_a_struct_is_refused() {
        assert!(SectionKeys::parse("[1, 2]").is_err());
        assert!(SectionKeys::parse("(accent: ").is_err());
    }

    #[test]
    fn enum_variants_and_payloads_survive_a_merge() {
        let merged = keys("(accent: Purple, mode: Pair(1, 2))")
            .overlaid(keys("(accent: Blue, mode: On(level: 2))"));
        let section: Section = merged.parse_value().unwrap_or_default();
        assert_eq!(section.accent, Accent::Blue);
        assert_eq!(section.mode, Mode::On { level: 2 });
    }

    #[test]
    fn an_absent_key_keeps_the_lower_layer_and_a_present_key_replaces_it_whole() {
        let merged = keys("(height: 40, inner: (first: 1, second: 2))")
            .overlaid(keys("(inner: (first: 3))"));
        let section: Section = merged.parse_value().unwrap_or_default();
        assert_eq!(section.height, Some(40));
        assert_eq!(
            section.inner,
            Inner {
                first: 3,
                second: None
            }
        );
    }

    #[test]
    fn a_user_value_equal_to_the_field_default_wins_over_a_distro_default() {
        let merged = keys("(accent: Blue)").overlaid(keys("(accent: Purple)"));
        let section: Section = merged.parse_value().unwrap_or_default();
        assert_eq!(section.accent, Accent::Purple);
    }

    #[test]
    fn an_explicit_none_replaces_a_lower_value() {
        let merged = keys("(height: 40)").overlaid(keys("(height: None)"));
        let section: Section = merged.parse_value().unwrap_or_default();
        assert_eq!(section.height, None);
    }

    #[test]
    fn the_written_text_parses_back_to_the_same_keys() {
        let canonical = SectionKeys::of_value(&Section::default()).unwrap_or_default();
        assert_eq!(keys(&canonical.to_ron()), canonical);
        assert_eq!(
            canonical.get("items").map(|items| items.contains('\n')),
            Some(true)
        );
    }

    #[test]
    fn equal_fields_give_equal_text() {
        let first = SectionKeys::of_value(&Section::default()).unwrap_or_default();
        let changed = Section {
            accent: Accent::Blue,
            ..Section::default()
        };
        let second = SectionKeys::of_value(&changed).unwrap_or_default();
        let differing: Vec<&str> = first
            .entries
            .iter()
            .map(|(name, _)| name.as_str())
            .filter(|key| first.get(key) != second.get(key))
            .collect();
        assert_eq!(differing, ["accent"]);
    }
}
