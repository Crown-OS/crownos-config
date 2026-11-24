use crate::patch::PatchError;
use crate::schema::{SECTIONS, check_value};
use crate::section_keys::SectionKeys;

/// Checks a default layer file of `section`: it must parse, and for a section
/// of this crate each key must exist and have a valid value. serde ignores an
/// unknown key, so without this check a misspelt default ships silently. A
/// section of another crate, such as `dock`, is only parsed.
pub fn validate_default_layer(section: &str, text: &str) -> Result<(), PatchError> {
    let keys = SectionKeys::parse(text).map_err(|error| PatchError::Unparseable {
        section: section.to_owned(),
        message: error.to_string(),
    })?;
    if !SECTIONS.contains(&section) {
        return Ok(());
    }
    keys.entries()
        .try_for_each(|(key, value)| check_value(section, key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_valid_distro_file_passes() {
        assert!(validate_default_layer("appearance", "(accent: Blue, bar_height: 36)").is_ok());
    }

    #[test]
    fn an_unknown_key_or_a_wrong_type_fails() {
        assert!(matches!(
            validate_default_layer("appearance", "(acent: Blue)"),
            Err(PatchError::UnknownKey { .. })
        ));
        assert!(matches!(
            validate_default_layer("appearance", "(bar_height: \"tall\")"),
            Err(PatchError::InvalidValue { .. })
        ));
    }

    #[test]
    fn a_section_of_another_crate_is_only_parsed() {
        assert!(validate_default_layer("dock", "(items: [\"web-browser\"])").is_ok());
        assert!(validate_default_layer("dock", "(items: [").is_err());
    }
}
