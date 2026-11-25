//! The text of a user's section file. Each key that the user set is a normal
//! line. Each other key is a `//` line with its current default, so the user
//! removes the `//` to change it. The header records a stamp of the defaults:
//! when an update changes a default, the next read writes the file again with
//! the new defaults in the comments and the same keys of the user.

use std::fmt::Write as _;

use crate::section_keys::SectionKeys;
use crate::util::hash_bytes;

/// The first line of each user file that this crate writes.
pub const USER_FILE_HEADER: &str =
    "// CrownOS settings. A line that starts with // shows the default value.";

const STAMP_PREFIX: &str = "// defaults-stamp: ";

/// The user file for `user_changes` over `defaults`.
pub(super) fn render(section: &str, defaults: &SectionKeys, user_changes: &SectionKeys) -> String {
    let stamp = defaults_stamp(section, defaults);
    let mut text = header(section, &stamp);
    text.push_str(&body(section, defaults, user_changes));
    text
}

/// True when a program wrote `text` for other defaults. A file without a stamp
/// was written by hand and stays as it is until a program saves the section.
pub(super) fn has_outdated_defaults(text: &str, section: &str, defaults: &SectionKeys) -> bool {
    written_stamp(text).is_some_and(|stamp| stamp != defaults_stamp(section, defaults))
}

fn header(section: &str, stamp: &str) -> String {
    format!(
        "{USER_FILE_HEADER}\n\
         // Remove the // in front of a setting to change it. Programs that save\n\
         // settings write this file again, so comments that you add are not kept.\n\
         // Defaults: /usr/share/crownos/defaults/{section}.ron\n\
         {STAMP_PREFIX}{stamp}\n"
    )
}

fn defaults_stamp(section: &str, defaults: &SectionKeys) -> String {
    let template = body(section, defaults, &SectionKeys::default());
    format!("{:016x}", hash_bytes(template.as_bytes()))
}

fn written_stamp(text: &str) -> Option<&str> {
    text.lines()
        .take_while(|line| line.starts_with("//"))
        .find_map(|line| line.strip_prefix(STAMP_PREFIX))
}

fn body(section: &str, defaults: &SectionKeys, user_changes: &SectionKeys) -> String {
    let layout = if defaults.is_empty() {
        user_changes
    } else {
        defaults
    };
    let docs = crate::schema::key_docs(section);
    let (open, close) = layout.brackets();
    let mut text = format!("{open}\n");
    for (index, (name, default_value)) in defaults.entries().enumerate() {
        if index > 0 {
            text.push('\n');
        }
        for line in summary_lines(docs, name) {
            let _ = writeln!(text, "    // {line}");
        }
        match user_changes.get(name) {
            Some(value) => text.push_str(&layout.entry_text(name, value)),
            None => text.push_str(&commented(&layout.entry_text(name, default_value))),
        }
    }
    for (name, value) in user_changes.entries() {
        if !defaults.contains(name) {
            text.push_str(&layout.entry_text(name, value));
        }
    }
    text.push(close);
    text.push('\n');
    text
}

const SUMMARY_WIDTH: usize = 72;

/// The first sentence of the key's doc comment, wrapped to [`SUMMARY_WIDTH`].
fn summary_lines(docs: &[(&str, &[&str])], key: &str) -> Vec<String> {
    let Some((_, doc)) = docs.iter().find(|(name, _)| *name == key) else {
        return Vec::new();
    };
    let paragraph = doc
        .iter()
        .map(|line| line.trim())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .replace("[`", "`")
        .replace("`]", "`");
    let sentence = paragraph
        .find(". ")
        .map_or(paragraph.as_str(), |end| &paragraph[..=end]);
    wrap(sentence)
}

fn wrap(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= SUMMARY_WIDTH => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

/// Each line gets `// ` after the indent of the entry, so removing `// `
/// gives the entry back.
fn commented(entry: &str) -> String {
    entry
        .lines()
        .map(|line| match line.strip_prefix("    ") {
            Some(rest) => format!("    // {rest}\n"),
            None => format!("// {line}\n"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Compositor, Sound};

    fn defaults_of<T: serde::Serialize>(value: &T) -> SectionKeys {
        SectionKeys::of_value(value).unwrap_or_default()
    }

    #[test]
    fn a_file_without_user_changes_means_the_defaults() {
        let defaults = defaults_of(&Sound::default());
        let text = render(Sound::SECTION, &defaults, &SectionKeys::default());

        let parsed = SectionKeys::parse(&text).unwrap_or_default();
        assert!(parsed.is_empty(), "{text}");
        assert!(text.contains("    // muted: false,\n"), "{text}");
    }

    #[test]
    fn user_changes_are_active_lines_in_the_place_of_their_key() {
        let defaults = defaults_of(&Sound::default());
        let user_changes = SectionKeys::parse("(muted: true)").unwrap_or_default();
        let text = render(Sound::SECTION, &defaults, &user_changes);

        assert_eq!(SectionKeys::parse(&text).unwrap_or_default(), user_changes);
        let muted = text.find("    muted: true,").unwrap_or(usize::MAX);
        let input = text.find("input_volume").unwrap_or(usize::MAX);
        assert!(input < muted, "{text}");
    }

    #[test]
    fn removing_the_comment_marks_of_a_long_value_gives_valid_ron() {
        let defaults = defaults_of(&Compositor::default());
        let text = render(Compositor::SECTION, &defaults, &SectionKeys::default());
        let window_rules = text
            .lines()
            .skip_while(|line| !line.starts_with("    // window_rules:"))
            .take_while(|line| line.starts_with("    // "))
            .map(|line| line.replacen("// ", "", 1))
            .collect::<Vec<_>>()
            .join("\n");

        let uncommented = format!("(\n{window_rules}\n)");
        let parsed: Compositor = SectionKeys::parse(&uncommented)
            .and_then(|keys| keys.parse_value())
            .unwrap_or_else(|error| panic!("{error}: {uncommented}"));
        assert_eq!(parsed.window_rules, Compositor::default().window_rules);
    }

    #[test]
    fn each_key_has_the_first_sentence_of_its_doc_above_it() {
        let defaults = defaults_of(&Compositor::default());
        let text = render(Compositor::SECTION, &defaults, &SectionKeys::default());

        assert!(
            text.contains(
                "    // Your own programs, started after `session_startup`.\n    // startup: [],"
            ),
            "{text}"
        );
    }

    #[test]
    fn a_summary_is_one_sentence_wrapped() {
        let docs: &[(&str, &[&str])] = &[(
            "key",
            &[
                " One sentence that is long enough to need a second line when it is wrapped here.",
                " Two.",
                "",
                " Other paragraph.",
            ],
        )];
        let lines = summary_lines(docs, "key");

        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines.iter().all(|line| line.len() <= SUMMARY_WIDTH));
        assert!(lines[1].ends_with("wrapped here."), "{lines:?}");
    }

    #[test]
    fn the_stamp_follows_the_defaults() {
        let defaults = defaults_of(&Sound::default());
        let text = render(Sound::SECTION, &defaults, &SectionKeys::default());
        assert!(!has_outdated_defaults(&text, Sound::SECTION, &defaults));

        let louder = defaults_of(&Sound {
            output_volume: 80.0,
            ..Sound::default()
        });
        assert!(has_outdated_defaults(&text, Sound::SECTION, &louder));
        assert!(!has_outdated_defaults(
            "(muted: true)\n",
            Sound::SECTION,
            &louder
        ));
    }
}
