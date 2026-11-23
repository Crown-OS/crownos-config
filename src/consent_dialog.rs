//! The consent dialog that every CrownOS component asks a privacy question with. A component
//! runs the root-owned dialog program itself and reads the answer from its output. No D-Bus
//! name is in the path, so no other process of the user can answer in the user's place. The
//! compositor keeps synthetic input away from this program's windows.
//!
//! The arguments and the output are those of xdg-desktop-portal-crownos (`chooser::ask` and
//! `dialog/src/alert.rs`):
//!
//! ```text
//! crownos-portal-dialog --icon access|warning --title T --message M --cancel DENY
//!     --choice grant=GRANT [--checkbox LABEL --initial true|false]
//!     [--list LABEL --item A --item B --initial INDEX]
//! ```
//!
//! The dialog prints `option VALUE` for each extra question, then `grant` when the user said
//! yes, and exits with success.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Where xdg-desktop-portal-crownos installs the dialog.
pub const DIALOG_EXECUTABLE: &str = "/usr/lib/crownos-portal-dialog";
/// A development build of the dialog for nested tests. Debug builds only.
#[cfg(debug_assertions)]
const DIALOG_OVERRIDE: &str = "CROWN_CONSENT_DIALOG";
const GRANT: &str = "grant";
const OPTION_PREFIX: &str = "option ";
const TRUE: &str = "true";
const FALSE: &str = "false";

/// The dialog program to run.
pub fn dialog_executable() -> PathBuf {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os(DIALOG_OVERRIDE) {
        return PathBuf::from(path);
    }
    PathBuf::from(DIALOG_EXECUTABLE)
}

/// An extra question below the main one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentOption {
    /// Answers `true` or `false`.
    Checkbox { label: String, initially_on: bool },
    /// Answers the index of the chosen item.
    List {
        label: String,
        items: Vec<String>,
        initial: usize,
    },
}

/// A yes/no question with its extra questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentQuestion {
    pub dangerous: bool,
    pub title: String,
    pub message: String,
    pub grant: String,
    pub deny: String,
    pub options: Vec<ConsentOption>,
}

/// What the user said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentAnswer {
    pub granted: bool,
    /// One value for each extra question, or `None` when the dialog did not print one for each
    /// (closed with Escape, or failed).
    pub option_values: Option<Vec<String>>,
}

impl ConsentAnswer {
    /// The checkbox at `index` was on.
    pub fn checked(&self, index: usize) -> bool {
        self.value(index) == Some(TRUE)
    }

    /// The item chosen in the list at `index`.
    pub fn chosen(&self, index: usize) -> Option<usize> {
        self.value(index)?.parse().ok()
    }

    fn value(&self, index: usize) -> Option<&str> {
        self.option_values.as_ref()?.get(index).map(String::as_str)
    }
}

impl ConsentQuestion {
    /// The dialog's arguments, in the portal's order.
    pub fn arguments(&self) -> Vec<String> {
        let icon = if self.dangerous { "warning" } else { "access" };
        let mut arguments: Vec<String> = [
            "--icon",
            icon,
            "--title",
            &self.title,
            "--message",
            &self.message,
            "--cancel",
            &self.deny,
            "--choice",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        arguments.push(format!("{GRANT}={}", self.grant));
        for option in &self.options {
            match option {
                ConsentOption::Checkbox {
                    label,
                    initially_on,
                } => arguments.extend([
                    "--checkbox".to_owned(),
                    label.clone(),
                    "--initial".to_owned(),
                    if *initially_on { TRUE } else { FALSE }.to_owned(),
                ]),
                ConsentOption::List {
                    label,
                    items,
                    initial,
                } => {
                    arguments.extend(["--list".to_owned(), label.clone()]);
                    for item in items {
                        arguments.extend(["--item".to_owned(), item.clone()]);
                    }
                    arguments.extend(["--initial".to_owned(), initial.to_string()]);
                }
            }
        }
        arguments
    }

    /// The dialog process, ready to run: no input, the answer on stdout.
    pub fn command(&self) -> Command {
        let mut command = Command::new(dialog_executable());
        command
            .args(self.arguments())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());
        command
    }

    /// The answer in the output of a dialog that exited with `succeeded`.
    pub fn answer(&self, succeeded: bool, stdout: &[u8]) -> ConsentAnswer {
        let text = std::str::from_utf8(stdout).unwrap_or_default();
        let granted = succeeded && text.lines().next_back().map(str::trim) == Some(GRANT);
        let values: Vec<String> = text
            .lines()
            .filter_map(|line| line.strip_prefix(OPTION_PREFIX))
            .map(|value| value.trim().to_owned())
            .collect();
        let option_values = values
            .len()
            .checked_sub(self.options.len())
            .and_then(|first| values.get(first..))
            .map(<[String]>::to_vec);
        ConsentAnswer {
            granted,
            option_values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question() -> ConsentQuestion {
        ConsentQuestion {
            dangerous: false,
            title: "Let notes use your screen context?".to_owned(),
            message: "exe:/usr/bin/notes asks what you work on.".to_owned(),
            grant: "Allow".to_owned(),
            deny: "Don't allow".to_owned(),
            options: vec![
                ConsentOption::Checkbox {
                    label: "Remember this choice".to_owned(),
                    initially_on: false,
                },
                ConsentOption::List {
                    label: "The app may see".to_owned(),
                    items: vec!["Window list".to_owned(), "Titles".to_owned()],
                    initial: 0,
                },
            ],
        }
    }

    #[test]
    fn the_arguments_are_the_portal_ones() {
        assert_eq!(
            question().arguments(),
            [
                "--icon",
                "access",
                "--title",
                "Let notes use your screen context?",
                "--message",
                "exe:/usr/bin/notes asks what you work on.",
                "--cancel",
                "Don't allow",
                "--choice",
                "grant=Allow",
                "--checkbox",
                "Remember this choice",
                "--initial",
                "false",
                "--list",
                "The app may see",
                "--item",
                "Window list",
                "--item",
                "Titles",
                "--initial",
                "0",
            ]
        );
    }

    #[test]
    fn a_grant_needs_success_and_the_grant_line() {
        let question = question();
        let answer = question.answer(true, b"option true\noption 1\ngrant\n");
        assert!(answer.granted && answer.checked(0));
        assert_eq!(answer.chosen(1), Some(1));
        assert!(
            !question
                .answer(false, b"option true\noption 1\ngrant\n")
                .granted
        );
        let denied = question.answer(false, b"option false\noption 0\n");
        assert!(!denied.granted && !denied.checked(0));
        let closed = question.answer(false, b"");
        assert_eq!(closed.option_values, None);
    }
}
